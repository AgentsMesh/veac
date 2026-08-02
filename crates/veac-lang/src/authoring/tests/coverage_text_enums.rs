use crate::authoring::{format_document, lower_document, parse};

fn source(old: &str, new: &str) -> String {
    let source = super::text::PROJECT;
    assert_eq!(
        source.matches(old).count(),
        1,
        "ambiguous replacement: {old}"
    );
    let mut source = source.replacen(old, new, 1);
    let path_incompatible = matches!(
        old,
        "wrap none;" | "overflow visible;" | "writing-mode horizontal-tb;" | "orientation upright;"
    ) && old != new;
    if path_incompatible {
        source = source.replace(
            r#"            path {
              point { x 10%; y 50%; }
              point { x 90%; y 50%; }
              start-offset 0px; reverse false; align start;
            }
"#,
            "",
        );
    }
    source
}

pub(super) fn text_style(old: &str, new: &str) -> veac_ir::TextStyle {
    let source = source(old, new);
    let document = parse(&source).unwrap_or_else(|error| panic!("{error}"));
    let formatted = format_document(&document);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    let envelope =
        lower_document(&document).unwrap_or_else(|error| panic!("{old} -> {new}: {error}"));
    let veac_ir::ClipSource::Text { style, .. } =
        &envelope.project.sequences[0].tracks[0].clips[0].source
    else {
        panic!("text source expected")
    };
    style.clone()
}

#[test]
fn every_font_weight_and_style_lowers_to_the_matching_enum() {
    use veac_ir::{FontStyle as S, FontWeight as W};
    for (token, expected) in [
        ("thin", W::Thin),
        ("extra-light", W::ExtraLight),
        ("light", W::Light),
        ("normal", W::Normal),
        ("medium", W::Medium),
        ("semi-bold", W::SemiBold),
        ("bold", W::Bold),
        ("extra-bold", W::ExtraBold),
        ("black", W::Black),
    ] {
        assert_eq!(
            text_style("weight bold;", &format!("weight {token};")).font_weight,
            expected
        );
    }
    for (token, expected) in [
        ("normal", S::Normal),
        ("italic", S::Italic),
        ("oblique", S::Oblique),
    ] {
        assert_eq!(
            text_style("font-style normal;", &format!("font-style {token};")).font_style,
            expected
        );
    }
}

#[test]
fn every_wrap_overflow_and_horizontal_alignment_mode_lowers_exactly() {
    use veac_ir::{HorizontalTextAlignment as H, TextOverflow as O, TextWrap as W};
    for (token, expected) in [
        ("none", W::None),
        ("word", W::Word),
        ("character", W::Character),
    ] {
        assert_eq!(
            text_style("wrap none;", &format!("wrap {token};"))
                .layout
                .wrap,
            expected
        );
    }
    for (token, expected) in [
        ("visible", O::Visible),
        ("clip", O::Clip),
        ("ellipsis", O::Ellipsis),
    ] {
        assert_eq!(
            text_style("overflow visible;", &format!("overflow {token};"))
                .layout
                .overflow,
            expected
        );
    }
    for (token, expected) in [
        ("left", H::Left),
        ("center", H::Center),
        ("right", H::Right),
    ] {
        assert_eq!(
            text_style(
                "horizontal-align center;",
                &format!("horizontal-align {token};"),
            )
            .layout
            .horizontal_alignment,
            expected
        );
    }
}

#[test]
fn vertical_writing_orientation_and_path_alignment_modes_lower_exactly() {
    use veac_ir::{
        TextOrientation as O, TextPathAlignment as P, TextWritingMode as W,
        VerticalTextAlignment as V,
    };
    for (token, expected) in [
        ("top", V::Top),
        ("middle", V::Middle),
        ("bottom", V::Bottom),
    ] {
        assert_eq!(
            text_style(
                "vertical-align middle;",
                &format!("vertical-align {token};"),
            )
            .layout
            .vertical_alignment,
            expected
        );
    }
    for (token, expected) in [
        ("horizontal-tb", W::HorizontalTb),
        ("vertical-rl", W::VerticalRl),
        ("vertical-lr", W::VerticalLr),
    ] {
        assert_eq!(
            text_style(
                "writing-mode horizontal-tb;",
                &format!("writing-mode {token};"),
            )
            .layout
            .writing_mode,
            expected
        );
    }
    for (token, expected) in [
        ("upright", O::Upright),
        ("sideways", O::Sideways),
        ("mixed", O::Mixed),
    ] {
        assert_eq!(
            text_style("orientation upright;", &format!("orientation {token};"))
                .layout
                .orientation,
            expected
        );
    }
    for (token, expected) in [("start", P::Start), ("center", P::Center), ("end", P::End)] {
        assert_eq!(
            text_style("align start;", &format!("align {token};"))
                .path
                .unwrap()
                .alignment,
            expected
        );
    }
}
