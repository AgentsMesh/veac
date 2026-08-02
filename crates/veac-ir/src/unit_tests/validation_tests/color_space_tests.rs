use super::*;

#[test]
fn rgb_color_spaces_require_full_range_in_canonical_pipelines() {
    let mut project = sample_project();
    let rgb_limited = ColorSpace {
        primaries: ColorPrimaries::Bt709,
        transfer: ColorTransfer::Srgb,
        matrix: ColorMatrix::Rgb,
        range: ColorRange::Limited,
    };
    project.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .color_pipeline = Some(ColorPipeline {
        input: rgb_limited,
        working: rgb_limited,
        output: rgb_limited,
        stages: vec![],
    });

    assert_code(&validation_codes(&project), "COLOR_SPACE");
    assert!(color_space_valid(ColorSpace {
        range: ColorRange::Full,
        ..rgb_limited
    }));
}

#[test]
fn pq_and_hlg_color_spaces_are_supported_conversion_domains() {
    for transfer in [ColorTransfer::Smpte2084, ColorTransfer::AribStdB67] {
        assert!(color_space_valid(ColorSpace {
            primaries: ColorPrimaries::Bt2020,
            transfer,
            matrix: ColorMatrix::Bt2020Ncl,
            range: ColorRange::Limited,
        }));
    }
}
