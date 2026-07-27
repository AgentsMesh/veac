use veac_plan::canonical::{
    Animatable, Interpolation, Keyframe, KeyframeId, PitchPolicy, Rational,
    SourceTimeInterpolation, SourceTimeSegment,
};
use veac_plan::ResolvedSourceTimeMap;

use super::audio::{audio_plan, clip};
use super::support::{bindings, emit_video_command};

#[test]
fn follow_speed_rejects_unexecutable_asetrate_before_graph_building() {
    for rate in [
        Rational::new(1, 100_000).unwrap(),
        Rational::new(i64::from(i32::MAX), 1).unwrap(),
    ] {
        let mut plan = audio_plan(2);
        let clip = clip(&mut plan);
        clip.audio.as_mut().unwrap().pitch_policy = PitchPolicy::FollowSpeed;
        let mapping = clip.source_mapping.as_mut().unwrap();
        let ResolvedSourceTimeMap::Linear { rate: value, .. } = &mut mapping.time_map else {
            unreachable!()
        };
        *value = rate;

        let error = emit_video_command(&plan, &bindings(&plan)).unwrap_err();

        assert!(error
            .diagnostics()
            .iter()
            .any(|value| value.code == "PLAN_AUDIO_RATE_INVALID"));
    }
}

#[test]
fn output_specific_sample_alignment_and_pan_fail_in_preflight() {
    let mut alignment = audio_plan(2);
    alignment
        .output
        .video_deliverable_mut()
        .unwrap()
        .audio
        .as_mut()
        .unwrap()
        .sample_rate = 44_100;
    clip(&mut alignment).record_range.start.value = 1;
    alignment.sequences[0].duration.value += 1;
    assert_code(&alignment, "PLAN_AUDIO_SAMPLE_ALIGNMENT_INVALID");

    let mut pan = audio_plan(1);
    pan.sequences[0].tracks[0].clips[0]
        .audio
        .as_mut()
        .unwrap()
        .pan = veac_plan::canonical::Animatable::constant(0.5);
    assert_code(&pan, "PLAN_AUDIO_CHANNEL_LAYOUT_INVALID");
}

#[test]
fn keyframed_pan_and_follow_speed_curves_pass_backend_rate_validation() {
    let mut pan = audio_plan(2);
    clip(&mut pan).audio.as_mut().unwrap().pan = Animatable::Keyframes {
        keyframes: vec![key("kf_pan_start", 0, 0.0), key("kf_pan_end", 300, 0.5)],
    };
    let graph = emit_video_command(&pan, &bindings(&pan))
        .unwrap()
        .filter_graph
        .unwrap();
    assert!(graph.contains("panleft"), "graph={graph}");

    let mut curve = audio_plan(2);
    let clip = clip(&mut curve);
    clip.audio.as_mut().unwrap().pitch_policy = PitchPolicy::FollowSpeed;
    let mapping = clip.source_mapping.as_mut().unwrap();
    mapping.time_map = ResolvedSourceTimeMap::Curve {
        segments: vec![SourceTimeSegment {
            record_duration: super::support::time(600),
            source_start: super::support::time(0),
            source_end: super::support::time(600),
            interpolation: SourceTimeInterpolation::Linear,
        }],
    };
    let graph = emit_video_command(&curve, &bindings(&curve))
        .unwrap()
        .filter_graph
        .unwrap();
    assert!(graph.contains("asetrate=48000*1"), "graph={graph}");
}

fn key(id: &str, at: i64, value: f64) -> Keyframe<f64> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: super::support::time(at),
        value,
        interpolation: Interpolation::Linear,
    }
}

fn assert_code(plan: &veac_plan::ResolvedRenderPlan, code: &str) {
    let error = emit_video_command(plan, &bindings(plan)).unwrap_err();
    assert!(error.diagnostics().iter().any(|value| value.code == code));
}
