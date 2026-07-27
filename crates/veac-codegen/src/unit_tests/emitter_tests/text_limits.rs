use veac_plan::canonical::*;
use veac_plan::ResolvedClipSource;

use super::support::{bindings, emit_video_command, resolved, text_fixture};

#[test]
fn inline_ass_and_final_filter_stay_below_linux_single_argument_limit() {
    let mut plan = resolved(&text_fixture(false));
    text_content(&mut plan).text = "a".repeat(40_000);
    text_content(&mut plan).style.background = None;
    let command = emit_video_command(&plan, &bindings(&plan)).unwrap();
    let graph = command.filter_graph.as_ref().unwrap();
    assert!(graph.len() < 120_000, "filter bytes={}", graph.len());
    assert!(
        command
            .to_args()
            .iter()
            .all(|argument| argument.len() < 131_072),
        "one structured argv element exceeds Linux MAX_ARG_STRLEN"
    );
}

#[test]
fn oversized_ass_still_fails_at_the_artifact_derived_payload_budget() {
    let mut plan = resolved(&text_fixture(false));
    plan.sequences[0].settings.frame_rate = Rational::new(30, 1).unwrap();
    plan.sequences[0].duration = RationalTime::new(3_600, 600).unwrap();
    plan.sequences[0].tracks[1].clips[0].record_range.duration =
        RationalTime::new(3_000, 600).unwrap();
    let content = text_content(&mut plan);
    content.text = "animated ".repeat(40);
    content.style.background = None;
    content.style.animation = Some(TextAnimation {
        granularity: TextGranularity::Grapheme,
        transform: TextUnitTransform::default(),
        reveal: Animatable::constant(1.0),
        highlight: None,
        opacity: Animatable::Keyframes {
            keyframes: vec![key("kf_payload_a", 0, 0.0), key("kf_payload_b", 600, 1.0)],
        },
        stagger: RationalTime::zero(600).unwrap(),
    });
    let error = emit_video_command(&plan, &bindings(&plan)).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "TEXT_ASS_PAYLOAD_LIMIT");
}

#[test]
fn untrusted_text_and_surface_budgets_fail_before_font_loading() {
    let mut plan = resolved(&text_fixture(false));
    let content = text_content(&mut plan);
    content.text = "a".repeat(MAX_TEXT_BYTES + 1);
    content.style.layout.box_width_pixels = Some(MAX_TEXT_BOX_DIMENSION);
    content.style.layout.box_height_pixels = Some(MAX_TEXT_BOX_DIMENSION);

    let error = emit_video_command(&plan, &bindings(&plan)).unwrap_err();

    assert!(error
        .diagnostics()
        .iter()
        .any(|value| value.code == "PLAN_TEXT_INVALID"));
}

#[test]
fn animated_ass_rejects_frame_rates_above_its_exact_timebase() {
    let mut plan = resolved(&text_fixture(false));
    plan.sequences[0].settings.frame_rate = Rational::new(101, 1).unwrap();
    text_content(&mut plan).style.background = None;
    text_content(&mut plan).style.animation = Some(TextAnimation {
        granularity: TextGranularity::Whole,
        transform: TextUnitTransform::default(),
        reveal: Animatable::constant(1.0),
        highlight: None,
        opacity: Animatable::Keyframes {
            keyframes: vec![key("kf_limit_a", 0, 0.0), key("kf_limit_b", 600, 1.0)],
        },
        stagger: RationalTime::zero(600).unwrap(),
    });
    let error = emit_video_command(&plan, &bindings(&plan)).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "TEXT_ASS_TIMEBASE_LIMIT");
}

#[test]
fn animated_ass_rejects_unbounded_sampling_before_allocation() {
    let mut plan = resolved(&text_fixture(false));
    let duration = RationalTime::new(1_000_000, 600).unwrap();
    plan.sequences[0].duration = RationalTime::new(1_000_600, 600).unwrap();
    plan.sequences[0].tracks[1].clips[0].record_range.duration = duration;
    text_content(&mut plan).style.background = None;
    text_content(&mut plan).style.animation = Some(TextAnimation {
        granularity: TextGranularity::Whole,
        transform: TextUnitTransform::default(),
        reveal: Animatable::constant(1.0),
        highlight: None,
        opacity: Animatable::Keyframes {
            keyframes: vec![key("kf_huge_a", 0, 0.0), key("kf_huge_b", 600, 1.0)],
        },
        stagger: RationalTime::zero(600).unwrap(),
    });

    let error = emit_video_command(&plan, &bindings(&plan)).unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "TEXT_ASS_EVENT_LIMIT");
}

#[test]
fn animated_ass_bounds_the_frame_by_unit_sample_matrix() {
    let mut plan = resolved(&text_fixture(false));
    let content = text_content(&mut plan);
    content.text = "a".repeat(10_000);
    content.style.background = None;
    content.style.animation = Some(TextAnimation {
        granularity: TextGranularity::Grapheme,
        transform: TextUnitTransform::default(),
        reveal: Animatable::constant(1.0),
        highlight: None,
        opacity: Animatable::Keyframes {
            keyframes: vec![key("kf_matrix_a", 0, 0.0), key("kf_matrix_b", 600, 1.0)],
        },
        stagger: RationalTime::zero(600).unwrap(),
    });

    let error = emit_video_command(&plan, &bindings(&plan)).unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "TEXT_ANIMATION_SAMPLE_LIMIT");
}

fn text_content(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut veac_plan::ResolvedText {
    match &mut plan.sequences[0].tracks[1].clips[0].source {
        ResolvedClipSource::Text { content } => content,
        _ => panic!("text fixture"),
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
