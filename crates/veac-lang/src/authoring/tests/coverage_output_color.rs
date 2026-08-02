use super::coverage_output_support::video;

fn color_space(primaries: &str, transfer: &str, matrix: &str, range: &str) -> veac_ir::ColorSpace {
    let color = format!(
        "color-space {{ primaries {primaries}; transfer {transfer}; matrix {matrix}; range {range}; }}"
    );
    let hdr = matches!(transfer, "smpte2084" | "arib-std-b67");
    let wide = primaries == "bt2020";
    let settings = if wide || hdr {
        format!("codec h265; pixel-format yuv420p10le; {color}")
    } else {
        color
    };
    video(&settings, "audio none;").video.color_space.unwrap()
}

#[test]
fn every_output_color_primary_lowers_to_the_matching_closed_enum() {
    use veac_ir::ColorPrimaries as C;
    for (token, expected) in [
        ("bt709", C::Bt709),
        ("bt470-m", C::Bt470M),
        ("bt470-bg", C::Bt470Bg),
        ("smpte170-m", C::Smpte170M),
        ("smpte240-m", C::Smpte240M),
        ("film", C::Film),
        ("bt2020", C::Bt2020),
        ("smpte428", C::Smpte428),
        ("smpte431", C::Smpte431),
        ("smpte432", C::Smpte432),
    ] {
        let matrix = if token == "bt2020" {
            "bt2020-ncl"
        } else {
            "bt709"
        };
        assert_eq!(
            color_space(token, "bt709", matrix, "limited").primaries,
            expected
        );
    }
}

#[test]
fn every_output_transfer_function_lowers_to_the_matching_closed_enum() {
    use veac_ir::ColorTransfer as C;
    for (token, expected) in [
        ("bt709", C::Bt709),
        ("gamma22", C::Gamma22),
        ("gamma28", C::Gamma28),
        ("smpte170-m", C::Smpte170M),
        ("smpte240-m", C::Smpte240M),
        ("linear", C::Linear),
        ("srgb", C::Srgb),
        ("bt2020-10", C::Bt2020_10),
        ("bt2020-12", C::Bt2020_12),
        ("smpte2084", C::Smpte2084),
        ("arib-std-b67", C::AribStdB67),
    ] {
        let (primaries, matrix) = if matches!(token, "smpte2084" | "arib-std-b67") {
            ("bt2020", "bt2020-ncl")
        } else {
            ("bt709", "bt709")
        };
        assert_eq!(
            color_space(primaries, token, matrix, "limited").transfer,
            expected
        );
    }
}

#[test]
fn every_output_matrix_and_range_lowers_to_the_matching_closed_enum() {
    use veac_ir::ColorMatrix as C;
    for (token, expected) in [
        ("rgb", C::Rgb),
        ("bt709", C::Bt709),
        ("fcc", C::Fcc),
        ("bt470-bg", C::Bt470Bg),
        ("smpte170-m", C::Smpte170M),
        ("smpte240-m", C::Smpte240M),
        ("ycgco", C::Ycgco),
        ("bt2020-ncl", C::Bt2020Ncl),
    ] {
        assert_eq!(
            color_space("bt709", "bt709", token, "limited").matrix,
            expected
        );
    }
    assert_eq!(
        color_space("bt709", "bt709", "bt709", "full").range,
        veac_ir::ColorRange::Full
    );
}
