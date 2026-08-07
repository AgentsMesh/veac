use veac_ir::{
    ClipSource, FontRef, FontStyle, FontWeight, HorizontalTextAlignment, TextOrientation,
    TextOverflow, TextPathAlignment, TextWrap, TextWritingMode, VerticalTextAlignment,
};

use super::{clips, envelope, style};

#[test]
fn chinese_example_publishes_all_text_sources_and_canonical_roundtrip() {
    let value = envelope();
    let clips = clips(&value);
    assert_eq!(clips.len(), 4);
    assert!(matches!(
        &clips[0].source,
        ClipSource::Text { text, .. } if text == "逐词动画：代码化视频"
    ));
    assert!(matches!(
        &clips[1].source,
        ClipSource::Text { text, .. } if text == "整体动画：路径文字"
    ));
    assert!(matches!(
        &clips[2].source,
        ClipSource::Caption { text, speaker: None, .. } if text == "逐字动画：中文可观察"
    ));
    assert!(matches!(
        &clips[3].source,
        ClipSource::Caption { text, speaker: Some(speaker), .. }
            if text == "逐行动画：布局与装饰" && speaker == "VEAC"
    ));
    assert!(veac_ir::validate(&value).is_ok());
    let json = veac_ir::canonical_json(&value).unwrap();
    assert_eq!(veac_ir::decode_canonical_json(&json).unwrap(), value);
}

#[test]
fn font_stack_spans_and_decorations_are_lossless_and_complete() {
    let value = envelope();
    let clips = clips(&value);
    let style = style(&clips[0].source);
    let FontRef::Material { material_id } = &style.font else {
        panic!("primary font must be the authored material")
    };
    assert_eq!(material_id, &value.project.materials[0].id);
    assert_eq!(
        style.fallback_fonts,
        ["PingFang SC", "Noto Sans CJK SC"].map(|family| FontRef::Family {
            family: family.to_owned()
        })
    );
    let span = &style.spans[0];
    assert_eq!((span.start, span.end), (0, 4));
    assert_eq!(
        span.font,
        Some(FontRef::Family {
            family: "sans-serif".to_owned()
        })
    );
    assert_eq!(span.font_weight, Some(FontWeight::ExtraBold));
    assert_eq!(span.font_style, Some(FontStyle::Italic));
    assert_eq!(span.size_pixels, Some(48.0));
    assert!(span.color.is_some());
    assert!(style.background.is_some());
    assert!(style.outline.is_some());
    assert!(style.shadow.is_some());
}

#[test]
fn box_axes_writing_orientation_alignment_and_path_remain_semantic() {
    let value = envelope();
    let clips = clips(&value);
    let horizontal = &style(&clips[0].source).layout;
    assert_eq!(
        (horizontal.box_width_pixels, horizontal.box_height_pixels),
        (Some(520.0), Some(120.0))
    );
    assert_eq!(horizontal.wrap, TextWrap::Word);
    assert_eq!(horizontal.overflow, TextOverflow::Ellipsis);
    assert_eq!(
        horizontal.horizontal_alignment,
        HorizontalTextAlignment::Center
    );
    assert_eq!(horizontal.vertical_alignment, VerticalTextAlignment::Middle);
    assert_eq!(horizontal.writing_mode, TextWritingMode::HorizontalTb);
    assert_eq!(horizontal.orientation, TextOrientation::Mixed);

    let path_style = style(&clips[1].source);
    assert_eq!(
        (
            path_style.layout.box_width_pixels,
            path_style.layout.box_height_pixels
        ),
        (None, Some(120.0))
    );
    assert_eq!(
        path_style.layout.horizontal_alignment,
        HorizontalTextAlignment::Right
    );
    assert_eq!(
        path_style.layout.vertical_alignment,
        VerticalTextAlignment::Bottom
    );
    assert_eq!(path_style.layout.orientation, TextOrientation::Sideways);
    let path = path_style.path.as_ref().unwrap();
    assert_eq!(path.points.len(), 3);
    assert_eq!(path.alignment, TextPathAlignment::Center);

    let right_to_left = &style(&clips[2].source).layout;
    let left_to_right = &style(&clips[3].source).layout;
    assert_eq!(right_to_left.writing_mode, TextWritingMode::VerticalRl);
    assert_eq!(right_to_left.orientation, TextOrientation::Upright);
    assert_eq!(left_to_right.writing_mode, TextWritingMode::VerticalLr);
    assert_eq!(left_to_right.orientation, TextOrientation::Mixed);
    for vertical in [right_to_left, left_to_right] {
        assert_eq!(
            (vertical.box_width_pixels, vertical.box_height_pixels),
            (Some(160.0), Some(260.0))
        );
        assert_eq!(vertical.wrap, TextWrap::None);
        assert_eq!(vertical.overflow, TextOverflow::Clip);
        assert_eq!(vertical.horizontal_alignment, HorizontalTextAlignment::Left);
        assert_eq!(vertical.vertical_alignment, VerticalTextAlignment::Top);
    }
}
