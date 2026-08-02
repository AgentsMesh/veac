mod fonts;

use veac_plan::canonical::*;
use veac_plan::ResolvedClipSource;

use super::support::{ass_script, bindings, emit_video_command, resolved, text_fixture};

#[test]
fn box_alignment_line_height_and_unit_alpha_are_encoded_in_ass() {
    let mut plan = resolved(&text_fixture(false));
    let content = text_content(&mut plan);
    content.styled_mut().unwrap().layout = box_layout(120.0, 60.0, TextOverflow::Clip);
    content.styled_mut().unwrap().line_height = 1.5;
    content.styled_mut().unwrap().animation = Some(TextAnimation {
        granularity: TextGranularity::Grapheme,
        transform: TextUnitTransform::default(),
        reveal: Animatable::constant(1.0),
        highlight: None,
        opacity: Animatable::Keyframes {
            keyframes: vec![key("kf_alpha_a", 0, 0.0), key("kf_alpha_b", 600, 1.0)],
        },
        stagger: RationalTime::new(30, 600).unwrap(),
    });
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    assert!(graph.contains("s=120x60"), "graph={graph}");
    let ass = ass_script(&graph);
    for marker in [
        "PlayResX: 120",
        "PlayResY: 60",
        "\\1a&HFF&",
        "\\3a&HFF&",
        "\\pos(",
    ] {
        assert!(ass.contains(marker), "missing {marker}: {ass}");
    }
    assert!(ass.matches("Dialogue: 1").count() > 2, "ass={ass}");
}

#[test]
fn tracking_weight_style_rich_spans_wrap_and_ellipsis_execute() {
    let mut plan = resolved(&text_fixture(false));
    let content = text_content(&mut plan);
    content.text = "alpha beta gamma delta".to_owned();
    content.styled_mut().unwrap().tracking_pixels = 2.0;
    content.styled_mut().unwrap().font_weight = FontWeight::Bold;
    content.styled_mut().unwrap().font_style = FontStyle::Italic;
    let primary = content.styled().unwrap().font.clone();
    content.styled_mut().unwrap().fallback_fonts.push(primary);
    content.styled_mut().unwrap().layout = TextLayout {
        wrap: TextWrap::Word,
        ..box_layout(80.0, 35.0, TextOverflow::Ellipsis)
    };
    content
        .styled_mut()
        .unwrap()
        .spans
        .push(veac_plan::ResolvedTextSpan {
            start: 0,
            end: 5,
            font: None,
            font_weight: Some(FontWeight::Black),
            font_style: None,
            size_pixels: Some(36.0),
            color: Some(Color {
                red: 255,
                green: 0,
                blue: 0,
                alpha: 255,
            }),
        });
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    let ass = ass_script(&graph);
    for marker in ["…", "\\fsp2", "\\b900", "\\i1", "\\1c&H0000FF&"] {
        assert!(ass.contains(marker), "missing {marker}: {ass}");
    }
    assert!(!graph.contains("PLAN_TEXT_BACKEND_UNSUPPORTED"));
}

#[test]
fn animated_ass_has_a_deterministic_event_budget() {
    let mut plan = resolved(&text_fixture(false));
    let clip = &mut plan.sequences[0].tracks[1].clips[0];
    clip.record_range.duration = RationalTime::new(30_000, 600).unwrap();
    let ResolvedClipSource::Text { content } = &mut clip.source else {
        panic!("text fixture")
    };
    content.styled_mut().unwrap().background = None;
    content.styled_mut().unwrap().animation = Some(TextAnimation {
        granularity: TextGranularity::Whole,
        transform: TextUnitTransform::default(),
        reveal: Animatable::constant(1.0),
        highlight: None,
        opacity: Animatable::Keyframes {
            keyframes: vec![key("kf_budget_a", 0, 0.0), key("kf_budget_b", 30_000, 1.0)],
        },
        stagger: RationalTime::zero(600).unwrap(),
    });
    plan.sequences[0].duration = RationalTime::new(30_600, 600).unwrap();
    plan.sequences[0].settings.frame_rate = Rational::new(100, 1).unwrap();
    let error = emit_video_command(&plan, &bindings(&plan)).unwrap_err();
    assert_eq!(
        error.diagnostics()[0].code,
        "TEXT_ASS_EVENT_LIMIT",
        "diagnostics={:?}",
        error.diagnostics()
    );
}

fn text_content(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut veac_plan::ResolvedText {
    match &mut plan.sequences[0].tracks[1].clips[0].source {
        ResolvedClipSource::Text { content } => content,
        _ => panic!("text fixture"),
    }
}

fn box_layout(width: f64, height: f64, overflow: TextOverflow) -> TextLayout {
    TextLayout {
        box_width_pixels: Some(width),
        box_height_pixels: Some(height),
        wrap: TextWrap::None,
        overflow,
        horizontal_alignment: HorizontalTextAlignment::Right,
        vertical_alignment: VerticalTextAlignment::Bottom,
        ..TextLayout::default()
    }
}

fn key(id: &str, at: i64, value: f64) -> Keyframe<f64> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: RationalTime::new(at, 600).unwrap(),
        value,
        interpolation: Interpolation::Linear,
    }
}
