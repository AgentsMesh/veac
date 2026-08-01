use veac_plan::canonical::*;
use veac_plan::{ResolvedInputKind, ResolvedSourceTimeMap};

use super::support::{bindings, emit_video_command, fixture, resolved, time};

#[test]
fn original_streams_emit_first_boundary_video_clone_and_audio_silence() {
    let plan = boundary_plan(-600, SourceOutOfRangePolicy::HoldFirst);
    let graph = graph(&plan);
    for marker in [
        "trim=start=0:duration=10,setpts=PTS-STARTPTS,tpad=start_duration=1:start_mode=clone",
        "atrim=start=0:duration=10,asetpts=PTS-STARTPTS,adelay=48000S:all=1",
    ] {
        assert!(graph.contains(marker), "missing {marker}: {graph}");
    }
}

#[test]
fn original_streams_emit_last_boundary_video_clone_and_audio_silence() {
    let plan = boundary_plan(6_000, SourceOutOfRangePolicy::HoldLast);
    let graph = graph(&plan);
    for marker in ["tpad=stop=-1:stop_mode=clone", "apad=pad_len=48000"] {
        assert!(graph.contains(marker), "missing {marker}: {graph}");
    }
}

#[test]
fn audio_curve_hold_is_silence_without_reading_a_boundary_sample() {
    let mut project = fixture();
    enable_audio(&mut project);
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    clip.audio = Some(audio());
    clip.source_mapping = Some(SourceMapping {
        time_map: SourceTimeMap::Curve {
            segments: vec![SourceTimeSegment {
                record_duration: time(600),
                source_start: time(-600),
                source_end: time(-600),
                interpolation: SourceTimeInterpolation::Hold,
            }],
        },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: SourceOutOfRangePolicy::HoldFirst,
    });
    let graph = graph(&resolved(&project));
    assert!(graph.contains("rampholda"), "{graph}");
    assert!(!graph.contains("apad=pad_len"), "{graph}");
}

#[test]
fn image_mapping_is_time_invariant_even_for_a_large_source_coordinate() {
    let mut plan = resolved(&fixture());
    plan.inputs[0].kind = ResolvedInputKind::Media {
        material_kind: MaterialKind::Image,
    };
    plan.inputs[0].audio = None;
    plan.inputs[0].video.as_mut().unwrap().duration = None;
    let mapping = plan.sequences[0].tracks[0].clips[0]
        .source_mapping
        .as_mut()
        .unwrap();
    let ResolvedSourceTimeMap::Linear {
        source_range_per_repeat,
        ..
    } = &mut mapping.time_map
    else {
        unreachable!()
    };
    source_range_per_repeat.start = time(60_000);
    let graph = graph(&plan);
    assert!(graph.contains("loop=loop=-1:size=1:start=0,trim=start=0"));
    assert!(!graph.contains("trim=start=100"), "{graph}");
}

fn boundary_plan(start: i64, policy: SourceOutOfRangePolicy) -> veac_plan::ResolvedRenderPlan {
    let mut project = fixture();
    enable_audio(&mut project);
    let clip = &mut project.project.sequences[0].tracks[0].clips[0];
    clip.audio = Some(audio());
    clip.source_mapping = Some(SourceMapping {
        time_map: SourceTimeMap::Linear {
            source_start: time(start),
            rate: Rational::new(1, 1).unwrap(),
            repeat: 1,
            direction: PlaybackDirection::Forward,
        },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: policy,
    });
    resolved(&project)
}

fn graph(plan: &veac_plan::ResolvedRenderPlan) -> String {
    emit_video_command(plan, &bindings(plan))
        .unwrap()
        .filter_graph
        .unwrap()
}

fn enable_audio(project: &mut ProjectEnvelope) {
    project.project.render_configs[0]
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
}

fn audio() -> AudioProperties {
    AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: vec![],
        crossfade: None,
    }
}
