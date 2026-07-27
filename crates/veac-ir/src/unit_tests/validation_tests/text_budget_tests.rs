use super::*;

#[test]
fn text_content_style_and_surface_budgets_fail_canonical_validation() {
    let mut project = sample_project();
    let ClipSource::Caption { text, style, .. } =
        &mut project.project.sequences[0].tracks[1].clips[0].source
    else {
        panic!("caption fixture")
    };
    *text = "a".repeat(MAX_TEXT_BYTES + 1);
    style.size_pixels = MAX_TEXT_SIZE_PIXELS + 1.0;
    style.tracking_pixels = MAX_TEXT_TRACKING_PIXELS + 1.0;
    style.line_height = MAX_TEXT_LINE_HEIGHT + 1.0;
    style.layout.box_width_pixels = Some(MAX_TEXT_BOX_DIMENSION);
    style.layout.box_height_pixels = Some(MAX_TEXT_BOX_DIMENSION);
    style.fallback_fonts = vec![style.font.clone(); MAX_FALLBACK_FONTS + 1];

    let codes = validation_codes(&project);

    assert_code(&codes, "TEXT_STYLE");
    assert_code(&codes, "TEXT_LAYOUT");
}
