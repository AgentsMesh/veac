use super::*;

#[test]
fn one_axis_text_boxes_are_canonical() {
    for (width, height, writing) in [
        (Some(320.0), None, TextWritingMode::HorizontalTb),
        (None, Some(180.0), TextWritingMode::VerticalRl),
        (None, Some(180.0), TextWritingMode::VerticalLr),
    ] {
        let project = with_layout(TextLayout {
            box_width_pixels: width,
            box_height_pixels: height,
            wrap: TextWrap::Word,
            overflow: TextOverflow::Ellipsis,
            writing_mode: writing,
            ..TextLayout::default()
        });
        validate(&project).unwrap();
    }
}

#[test]
fn wrap_and_non_visible_overflow_require_the_inline_axis() {
    for (width, height, writing) in [
        (None, Some(180.0), TextWritingMode::HorizontalTb),
        (Some(320.0), None, TextWritingMode::VerticalRl),
    ] {
        for (wrap, overflow) in [
            (TextWrap::Word, TextOverflow::Visible),
            (TextWrap::None, TextOverflow::Clip),
            (TextWrap::None, TextOverflow::Ellipsis),
        ] {
            let codes = validation_codes(&with_layout(TextLayout {
                box_width_pixels: width,
                box_height_pixels: height,
                wrap,
                overflow,
                writing_mode: writing,
                ..TextLayout::default()
            }));
            assert_code(&codes, "TEXT_LAYOUT");
        }
    }
}

#[test]
fn one_axis_dimensions_still_require_positive_finite_bounds() {
    for value in [0.0, -1.0, f64::INFINITY, MAX_TEXT_BOX_DIMENSION + 1.0] {
        let codes = validation_codes(&with_layout(TextLayout {
            box_width_pixels: Some(value),
            ..TextLayout::default()
        }));
        assert_code(&codes, "TEXT_LAYOUT");
    }
}

fn with_layout(layout: TextLayout) -> ProjectEnvelope {
    let mut project = sample_project();
    let ClipSource::Caption { style, .. } =
        &mut project.project.sequences[0].tracks[1].clips[0].source
    else {
        panic!("caption fixture")
    };
    style.layout = layout;
    project
}
