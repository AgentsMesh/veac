use veac_ir::FontRef;

use super::{
    assert_animation_error, assert_text_error, assert_time_error, build, clips, envelope_source,
    style, SOURCE,
};

#[test]
fn font_resources_stacks_and_family_text_fail_closed_without_loss() {
    for source in [
        change("let font = font_resource(", "let font = image_resource("),
        change(
            "[font_family(\"PingFang SC\"), font_family(\"Noto Sans CJK SC\")]",
            "[font_family(\"PingFang SC\"), font_family(\"PingFang SC\")]",
        ),
        change("font_family(\"PingFang SC\")", "font_family(\"   \")"),
    ] {
        assert_text_error(&source);
    }

    let source = change(
        "font_family(\"PingFang SC\")",
        "font_family(\"  PingFang SC  \")",
    );
    let value = envelope_source(&source);
    assert_eq!(
        style(&clips(&value)[0].source).fallback_fonts[0],
        FontRef::Family {
            family: "  PingFang SC  ".to_owned()
        }
    );
}

#[test]
fn metrics_and_box_axis_contracts_reject_invalid_values() {
    for source in [
        change("42px, 1px, 1.2", "0px, 1px, 1.2"),
        change("42px, 1px, 1.2", "42px, 1px, 0.0"),
        change("text_box_fixed(520px, 120px)", "text_box_fixed(0px, 120px)"),
        change("text_box_fixed(160px, 260px)", "text_box_width(160px)"),
        change("text_box_height(120px)", "text_box_height(0px)"),
    ] {
        assert_text_error(&source);
    }
}

#[test]
fn malformed_spans_and_paths_are_rejected_before_ir_publication() {
    for spans in [
        vec![span(4, 4)],
        vec![span(0, 999)],
        vec![span(2, 4), span(0, 2)],
        vec![span(0, 3), span(2, 4)],
    ] {
        assert_text_error(&replace_spans(&spans.join(", ")));
    }
    for source in [
        change(
            "[point(40px, 180px), point(320px, 120px), point(600px, 180px)]",
            "[point(40px, 180px)]",
        ),
        change(
            "[point(40px, 180px), point(320px, 120px), point(600px, 180px)]",
            "[point(40px, 180px), point(40px, 180px)]",
        ),
        change(
            "writing_horizontal_tb(), orientation_sideways()",
            "writing_vertical_rl(), orientation_sideways()",
        ),
    ] {
        assert_text_error(&source);
    }
}

#[test]
fn animation_lists_interpolation_bounds_and_clip_duration_fail_closed() {
    let empty = between(
        "point_keyframes([",
        "]),\n      vector_constant",
        "point_keyframes([]),\n      vector_constant",
    );
    for source in [
        empty,
        change(
            "identifier(\"position-end\")",
            "identifier(\"position-start\")",
        ),
        change(
            "identifier(\"position-end\"), 800ms",
            "identifier(\"position-end\"), 0s",
        ),
        change(
            "interpolation_spring(3.0, 4.0, 0.0)",
            "interpolation_spring(0.0, 4.0, 0.0)",
        ),
        change(
            "interpolation_cubic_bezier(0.2, -0.4, 0.8, 1.4)",
            "interpolation_cubic_bezier(-0.1, -0.4, 0.8, 1.4)",
        ),
        change(
            "percent_constant(100%),\n    60ms",
            "percent_constant(101%),\n    60ms",
        ),
    ] {
        assert_animation_error(&source);
    }
    assert_text_error(&change(
        "identifier(\"position-end\"), 800ms",
        "identifier(\"position-end\"), 3s",
    ));
    assert_time_error(&change("    60ms\n  ))", "    -1ms\n  ))"));
}

#[test]
fn spring_scale_extrema_and_text_track_affinity_fail_closed() {
    let scale = r#"vector_keyframes([
        vector_keyframe(identifier("start"), 0s, vector(1.0, 1.0), interpolation_spring(3.0, 4.0, 0.0)),
        vector_keyframe(identifier("end"), 800ms, vector(16.0, 16.0), interpolation_linear())
      ])"#;
    assert_text_error(&change("vector_constant(vector(1.0, 1.0))", scale));
    for source in [
        change("let visual = visual_layer(", "let visual = audio_layer("),
        change(
            "let captions = caption_layer(",
            "let captions = visual_layer(",
        ),
    ] {
        assert!(build(&source).is_err());
    }
}

fn change(from: &str, to: &str) -> String {
    assert!(SOURCE.contains(from), "missing fixture fragment: {from}");
    SOURCE.replacen(from, to, 1)
}

fn between(start: &str, end: &str, replacement: &str) -> String {
    let begin = SOURCE.find(start).unwrap();
    let finish = SOURCE[begin..].find(end).unwrap() + begin + end.len();
    let mut source = SOURCE.to_owned();
    source.replace_range(begin..finish, replacement);
    source
}

fn replace_spans(spans: &str) -> String {
    between(
        "    [\n      text_span(0, 4",
        "    ],\n    reveal_motion(granularity)",
        &format!("    [{spans}],\n    reveal_motion(granularity)"),
    )
}

fn span(start: u32, end: u32) -> String {
    format!(
        "text_span({start}, {end}, text_run_style(font_family(\"sans-serif\"), \
         weight_bold(), font_style_normal(), 48px, #ffffffff))"
    )
}
