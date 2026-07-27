use veac_codegen::emitter::emit_all;
use veac_plan::canonical::*;
use veac_plan::ResolvedRenderPlan;

use super::support::{bindings, fixture, resolved};

#[test]
fn mutated_video_facts_cannot_exceed_coded_display_or_rotation_budgets() {
    let cases: [fn(&mut VideoStreamInfo); 4] = [
        |video| video.width = MAX_DIMENSION + 1,
        |video| {
            video.width = MAX_DIMENSION;
            video.height = MAX_DIMENSION;
        },
        |video| video.sample_aspect_ratio = Rational::new(16, 1).unwrap(),
        |video| {
            video.width = 3_000;
            video.height = 3_000;
            video.rotation_degrees = 45;
        },
    ];
    for mutate in cases {
        let mut plan = resolved(&fixture());
        mutate(&mut plan.inputs[0].video.as_mut().unwrap().info);
        assert_input_invalid(&plan);
    }
}

#[test]
fn mutated_audio_facts_cannot_exceed_decode_resource_budgets() {
    let cases: [fn(&mut AudioStreamInfo); 4] = [
        |audio| audio.sample_rate = MAX_INPUT_AUDIO_SAMPLE_RATE + 1,
        |audio| audio.channels = MAX_INPUT_AUDIO_CHANNELS + 1,
        |audio| audio.channel_layout = "stereo\ninvalid".to_owned(),
        |audio| audio.channel_layout = "x".repeat(MAX_CHANNEL_LAYOUT_BYTES + 1),
    ];
    for mutate in cases {
        let mut plan = resolved(&fixture());
        mutate(&mut plan.inputs[0].audio.as_mut().unwrap().info);
        assert_input_invalid(&plan);
    }
}

fn assert_input_invalid(plan: &ResolvedRenderPlan) {
    let error = emit_all(plan, &bindings(plan)).unwrap_err();
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code == "PLAN_INPUT_FACTS_INVALID"));
}
