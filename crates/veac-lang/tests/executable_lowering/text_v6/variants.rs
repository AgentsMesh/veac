use veac_ir::{Animatable, FontWeight, TextOverflow, TextPathAlignment, TextWrap};

use super::{clips, envelope_source, style, SOURCE};

#[test]
fn auto_and_fixed_boxes_preserve_authored_axes() {
    let auto = changed("text_box_fixed(520px, 120px)", "text_box_auto()");
    let auto = replace(
        &auto,
        "text_wrap_word(), text_overflow_ellipsis()",
        "text_wrap_none(), text_overflow_visible()",
    );
    let value = envelope_source(&auto);
    let layout = &style(&clips(&value)[0].source).layout;
    assert_eq!(
        (layout.box_width_pixels, layout.box_height_pixels),
        (None, None)
    );
    assert_eq!(
        (layout.wrap, layout.overflow),
        (TextWrap::None, TextOverflow::Visible)
    );

    let width = changed("text_box_fixed(520px, 120px)", "text_box_width(520px)");
    let width = replace(
        &width,
        "text_wrap_word(), text_overflow_ellipsis()",
        "text_wrap_word(), text_overflow_visible()",
    );
    let value = envelope_source(&width);
    let layout = &style(&clips(&value)[0].source).layout;
    assert_eq!(
        (layout.box_width_pixels, layout.box_height_pixels),
        (Some(520.0), None)
    );
}

#[test]
fn decoration_absence_is_not_materialized_as_a_default() {
    let variants = [
        (
            "text_background_present(#102030cc, 12px)",
            "text_background_none()",
            0,
        ),
        (
            "text_outline_present(#48d7caff, 2px)",
            "text_outline_none()",
            1,
        ),
        (
            "shadow_present(8px, 55%, vector(4.0, 6.0), #000000cc)",
            "shadow_none()",
            2,
        ),
    ];
    for (from, to, field) in variants {
        let value = envelope_source(&changed(from, to));
        let style = style(&clips(&value)[0].source);
        let absent = [
            style.background.is_none(),
            style.outline.is_none(),
            style.shadow.is_none(),
        ];
        assert!(absent[field]);
    }
}

#[test]
fn every_font_weight_reaches_the_closed_canonical_enum() {
    let variants = [
        ("weight_thin()", FontWeight::Thin),
        ("weight_extra_light()", FontWeight::ExtraLight),
        ("weight_light()", FontWeight::Light),
        ("weight_normal()", FontWeight::Normal),
        ("weight_semi_bold()", FontWeight::SemiBold),
        ("weight_black()", FontWeight::Black),
    ];
    for (constructor, expected) in variants {
        let replacement = format!("metrics(fonts, {constructor}, font_style_normal())");
        let source = changed(
            "metrics(fonts, weight_bold(), font_style_normal())",
            &replacement,
        );
        let value = envelope_source(&source);
        assert_eq!(style(&clips(&value)[0].source).font_weight, expected);
    }
}

#[test]
fn path_alignment_offset_and_direction_are_semantic() {
    for (constructor, expected) in [
        ("text_path_align_start()", TextPathAlignment::Start),
        ("text_path_align_end()", TextPathAlignment::End),
    ] {
        let value = envelope_source(&changed("text_path_align_center()", constructor));
        let path = style(&clips(&value)[1].source).path.as_ref().unwrap();
        assert_eq!(path.alignment, expected);
    }
    let source = changed(
        "286px, false, text_path_align_center()",
        "8px, true, text_path_align_start()",
    );
    let value = envelope_source(&source);
    let path = style(&clips(&value)[1].source).path.as_ref().unwrap();
    assert_eq!(path.start_offset.value, 8.0);
    assert!(path.reverse);
}

#[test]
fn optional_animation_parts_and_curve_shapes_remain_typed() {
    let value = envelope_source(&changed(
        "reveal_motion(granularity)",
        "text_animation_none()",
    ));
    assert!(style(&clips(&value)[0].source).animation.is_none());

    let value = envelope_source(&changed(
        "text_highlight_present(#ffe066ff, percent_constant(100%))",
        "text_highlight_none()",
    ));
    assert!(style(&clips(&value)[0].source)
        .animation
        .as_ref()
        .unwrap()
        .highlight
        .is_none());

    let source = between(
        SOURCE,
        "      point_keyframes([",
        "      vector_constant(vector(1.0, 1.0)),",
        "      point_constant(point(0px, 0px)),\n      vector_keyframes([\n        vector_keyframe(identifier(\"scale-start\"), 0s, vector(1.0, 1.0), interpolation_linear()),\n        vector_keyframe(identifier(\"scale-end\"), 800ms, vector(1.2, 0.8), interpolation_linear())\n      ]),",
    );
    let value = envelope_source(&source);
    let transform = &style(&clips(&value)[0].source)
        .animation
        .as_ref()
        .unwrap()
        .transform;
    assert!(matches!(
        transform.position_offset,
        Animatable::Constant { .. }
    ));
    assert_eq!(transform.scale.keyframes().unwrap().len(), 2);
}

fn changed(from: &str, to: &str) -> String {
    replace(SOURCE, from, to)
}

fn replace(source: &str, from: &str, to: &str) -> String {
    assert!(source.contains(from), "missing fixture fragment: {from}");
    source.replacen(from, to, 1)
}

fn between(source: &str, start: &str, end: &str, replacement: &str) -> String {
    let begin = source.find(start).unwrap();
    let finish = source[begin..].find(end).unwrap() + begin + end.len();
    let mut value = source.to_owned();
    value.replace_range(begin..finish, replacement);
    value
}
