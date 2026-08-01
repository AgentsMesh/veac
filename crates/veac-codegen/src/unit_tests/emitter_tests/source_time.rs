use veac_codegen::emitter::CodegenErrorKind;
use veac_plan::canonical::*;
use veac_plan::{ResolvedClipSource, ResolvedSourceTimeMap};

use super::support::{bindings, emit_video_command, fixture, resolved, time};

#[test]
fn ramp_and_hold_emit_segmented_video_and_audio_filters() {
    let mut project = fixture();
    project.project.render_configs[0]
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 1,
    });
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    clip.source_mapping = Some(curve());
    clip.audio = Some(AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: vec![],
        crossfade: None,
    });
    let plan = resolved(&project);
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    for marker in [
        "setpts=PTS*200/400",
        "rampholdv",
        "fps=fps=30/1:start_time=0.666666666667:round=down:eof_action=pass",
        "concat=n=3:v=1:a=0",
        "mi_mode=blend",
        "atempo=2",
        "rampholda",
        "atempo=0.5",
        "concat=n=3:v=0:a=1",
    ] {
        assert!(graph.contains(marker), "missing {marker}: {graph}");
    }
}

#[test]
fn frame_hold_sampling_uses_canvas_rate_instead_of_the_project_timebase() {
    let mut plan = resolved(&fixture());
    let source = plan.sequences[0].tracks[0].clips[0].source.clone();
    let ResolvedClipSource::Media {
        input_id,
        video_stream: Some(video_stream),
        ..
    } = source
    else {
        panic!("media source");
    };
    plan.sequences[0].tracks[0].clips[0].source = ResolvedClipSource::FreezeFrame {
        input_id,
        video_stream,
        source_time: RationalTime::new(300, 600).unwrap(),
    };
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    assert!(graph.contains("fps=fps=30/1:start_time=0.5:round=down"));
    assert!(!graph.contains("fps=fps=600"));
}

#[test]
fn malformed_resolved_curve_fails_closed() {
    let mut plan = resolved(&fixture());
    let mapping = plan.sequences[0].tracks[0].clips[0]
        .source_mapping
        .as_mut()
        .unwrap();
    mapping.time_map = ResolvedSourceTimeMap::Curve { segments: vec![] };
    let error = emit_video_command(&plan, &bindings(&plan)).unwrap_err();
    assert_eq!(error.diagnostics()[0].kind, CodegenErrorKind::InvalidPlan);
    assert_eq!(error.diagnostics()[0].code, "PLAN_SOURCE_TIME_MAP_INVALID");
}

fn curve() -> SourceMapping {
    SourceMapping {
        time_map: SourceTimeMap::Curve {
            segments: vec![
                segment(200, 0, 400, SourceTimeInterpolation::Linear),
                segment(100, 400, 400, SourceTimeInterpolation::Hold),
                segment(300, 400, 550, SourceTimeInterpolation::Linear),
            ],
        },
        frame_synthesis: FrameSynthesisPolicy::Blend,
        out_of_range: SourceOutOfRangePolicy::Strict,
    }
}

fn segment(
    duration: i64,
    start: i64,
    end: i64,
    interpolation: SourceTimeInterpolation,
) -> SourceTimeSegment {
    SourceTimeSegment {
        record_duration: time(duration),
        source_start: time(start),
        source_end: time(end),
        interpolation,
    }
}
