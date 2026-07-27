use veac_plan::canonical::*;
use veac_plan::ResolvedClipSource;

use super::support::{ass_script, bindings, emit_video_command, resolved, text_fixture};

#[test]
fn every_text_animation_granularity_executes_with_reveal_opacity_and_stagger() {
    for granularity in [
        TextGranularity::Whole,
        TextGranularity::Line,
        TextGranularity::Word,
        TextGranularity::Grapheme,
    ] {
        let mut plan = resolved(&text_fixture(false));
        let content = text_content(&mut plan);
        content.text = "one two\nthree".to_owned();
        content.style.background = None;
        content.style.animation = Some(TextAnimation {
            granularity,
            transform: TextUnitTransform::default(),
            reveal: Animatable::Keyframes {
                keyframes: vec![
                    key("kf_reveal_start", 0, 0.0),
                    key("kf_reveal_end", 300, 1.0),
                ],
            },
            highlight: None,
            opacity: Animatable::Keyframes {
                keyframes: vec![
                    key("kf_opacity_start", 0, 0.0),
                    key("kf_opacity_end", 300, 1.0),
                ],
            },
            stagger: RationalTime::new(30, 600).unwrap(),
        });
        let graph = emit_video_command(&plan, &bindings(&plan))
            .unwrap()
            .filter_graph
            .unwrap();
        let ass = ass_script(&graph);
        assert!(ass.contains("\\1a&HFF&"), "{granularity:?}: {ass}");
        assert!(ass.contains("\\1a&H00&"), "{granularity:?}: {ass}");
        assert!(
            ass.matches("Dialogue: 1").count() >= 2,
            "{granularity:?}: {ass}"
        );
    }
}

#[test]
fn highlight_progress_overrides_only_completed_unit_fills() {
    let mut plan = resolved(&text_fixture(false));
    let content = text_content(&mut plan);
    content.text = "one two".to_owned();
    content.style.background = None;
    content.style.animation = Some(TextAnimation {
        granularity: TextGranularity::Word,
        transform: TextUnitTransform::default(),
        reveal: Animatable::constant(1.0),
        highlight: Some(TextHighlightAnimation {
            fill: Color {
                red: 255,
                green: 0,
                blue: 0,
                alpha: 255,
            },
            progress: Animatable::constant(0.5),
        }),
        opacity: Animatable::constant(1.0),
        stagger: RationalTime::zero(600).unwrap(),
    });
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    let ass = ass_script(&graph);
    assert!(ass.contains("\\1c&H0000FF&"), "{ass}");
    assert!(ass.contains("\\1c&H14F0FF&"), "{ass}");
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
