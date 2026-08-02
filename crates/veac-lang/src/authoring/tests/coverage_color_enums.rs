use crate::authoring::{format_document, lower_document, parse};

fn color(document: &mut crate::authoring::Document) -> &mut crate::authoring::ColorModifierDecl {
    document.project.sequences[0].layers[0].items[0]
        .modifiers
        .iter_mut()
        .find_map(|value| match value {
            crate::authoring::ModifierDecl::Color(value) => Some(value),
            _ => None,
        })
        .unwrap()
}

fn lower(old: &str, new: &str) -> Result<veac_ir::ColorPipeline, crate::authoring::Diagnostics> {
    assert_eq!(super::color::PROJECT.matches(old).count(), 1);
    let source = super::color::PROJECT.replacen(old, new, 1);
    let document = parse(&source).unwrap_or_else(|error| panic!("{error}"));
    let formatted = format_document(&document);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    let envelope = lower_document(&document)?;
    Ok(envelope.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_ref()
        .unwrap()
        .color_pipeline
        .clone()
        .unwrap())
}

fn pipeline(old: &str, new: &str) -> veac_ir::ColorPipeline {
    lower(old, new).unwrap_or_else(|error| panic!("{old} -> {new}: {error}"))
}

#[test]
fn every_hsl_range_lowers_to_the_matching_closed_enum() {
    use veac_ir::HueRange as H;
    for (token, expected) in [
        ("red", H::Red),
        ("yellow", H::Yellow),
        ("green", H::Green),
        ("cyan", H::Cyan),
        ("blue", H::Blue),
        ("magenta", H::Magenta),
    ] {
        let value = pipeline("range red;", &format!("range {token};"));
        let veac_ir::ColorStage::Hsl { adjustment } = value.stages[2] else {
            panic!("hsl stage expected")
        };
        assert_eq!(adjustment.range, expected);
    }
}

#[test]
fn every_curve_channel_and_interpolation_lowers_exactly() {
    for token in ["luma", "red", "green", "blue"] {
        let value = pipeline("curve luma {", &format!("curve {token} {{"));
        let veac_ir::ColorStage::Curves { curves } = &value.stages[3] else {
            panic!("curve stage expected")
        };
        let selected = match token {
            "luma" => &curves.luma,
            "red" => &curves.red,
            "green" => &curves.green,
            "blue" => &curves.blue,
            _ => unreachable!(),
        };
        assert!(selected.is_some());
    }
    let value = pipeline("interpolation natural;", "interpolation monotonic;");
    let veac_ir::ColorStage::Curves { curves } = &value.stages[3] else {
        panic!("curve stage expected")
    };
    assert_eq!(
        curves.luma.as_ref().unwrap().interpolation,
        veac_ir::ToneCurveInterpolation::Monotonic
    );
}

#[test]
fn every_lut_interpolation_lowers_to_the_matching_closed_enum() {
    use veac_ir::LutInterpolation as L;
    for (token, expected, valid_for_cube_3d) in [
        ("nearest", L::Nearest, true),
        ("linear", L::Linear, false),
        ("cosine", L::Cosine, false),
        ("cubic", L::Cubic, false),
        ("spline", L::Spline, false),
        ("trilinear", L::Trilinear, true),
        ("tetrahedral", L::Tetrahedral, true),
        ("pyramid", L::Pyramid, true),
        ("prism", L::Prism, true),
    ] {
        let replacement = format!("interpolation {token};");
        if !valid_for_cube_3d {
            let error = lower("interpolation trilinear;", &replacement).unwrap_err();
            assert!(error.as_slice().iter().any(|value| {
                value.code == "AUTHORING_LOWER_IR_VALIDATION"
                    && value.message.contains("LUT_MATERIAL_KIND")
            }));
            continue;
        }
        let value = pipeline("interpolation trilinear;", &replacement);
        let veac_ir::ColorStage::Lut { application } = &value.stages[5] else {
            panic!("lut stage expected")
        };
        assert_eq!(application.interpolation, expected);
    }
}

#[test]
fn formatter_emits_every_authored_color_space_keyword() {
    use veac_ir::{ColorMatrix as M, ColorPrimaries as P, ColorTransfer as T};
    let mut document = parse(super::color::PROJECT).unwrap();
    for (value, token) in [
        (P::Bt470M, "bt470-m"),
        (P::Bt470Bg, "bt470-bg"),
        (P::Smpte170M, "smpte170-m"),
        (P::Smpte240M, "smpte240-m"),
        (P::Film, "film"),
        (P::Bt2020, "bt2020"),
        (P::Smpte428, "smpte428"),
        (P::Smpte431, "smpte431"),
        (P::Smpte432, "smpte432"),
    ] {
        color(&mut document).input.value.primaries = value;
        assert!(format_document(&document).contains(&format!("primaries {token};")));
    }
    for (value, token) in [
        (T::Gamma22, "gamma22"),
        (T::Gamma28, "gamma28"),
        (T::Smpte170M, "smpte170-m"),
        (T::Smpte240M, "smpte240-m"),
        (T::Linear, "linear"),
        (T::Srgb, "srgb"),
        (T::Bt2020_10, "bt2020-10"),
        (T::Bt2020_12, "bt2020-12"),
        (T::Smpte2084, "smpte2084"),
        (T::AribStdB67, "arib-std-b67"),
    ] {
        color(&mut document).input.value.transfer = value;
        assert!(format_document(&document).contains(&format!("transfer {token};")));
    }
    for (value, token) in [
        (M::Rgb, "rgb"),
        (M::Fcc, "fcc"),
        (M::Bt470Bg, "bt470-bg"),
        (M::Smpte170M, "smpte170-m"),
        (M::Smpte240M, "smpte240-m"),
        (M::Ycgco, "ycgco"),
        (M::Bt2020Ncl, "bt2020-ncl"),
    ] {
        color(&mut document).input.value.matrix = value;
        assert!(format_document(&document).contains(&format!("matrix {token};")));
    }
}
