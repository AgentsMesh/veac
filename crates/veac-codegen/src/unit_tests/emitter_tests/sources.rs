use veac_codegen::emitter::CodegenErrorKind;
use veac_plan::canonical::*;
use veac_plan::{ResolvedClipSource, ResolvedInputKind, ResolvedSourceTimeMap};

use super::support::{bindings, emit_video_command, fixture, resolved, time};

#[test]
fn media_source_emits_reverse_repeat_speed_sampling_sar_and_rotation() {
    for (sampling, marker) in [
        (FrameSynthesisPolicy::Blend, "mi_mode=blend"),
        (FrameSynthesisPolicy::MotionCompensated, "mi_mode=mci"),
    ] {
        let mut plan = resolved(&fixture());
        let mapping = clip(&mut plan).source_mapping.as_mut().unwrap();
        let ResolvedSourceTimeMap::Linear {
            rate,
            direction,
            repeat,
            ..
        } = &mut mapping.time_map
        else {
            panic!("linear mapping");
        };
        *direction = PlaybackDirection::Reverse;
        *repeat = 2;
        *rate = Rational::new(2, 1).unwrap();
        mapping.frame_synthesis = sampling;
        let info = &mut plan.inputs[0].video.as_mut().unwrap().info;
        info.sample_aspect_ratio = Rational::new(2, 1).unwrap();
        info.rotation_degrees = 90;
        let graph = graph(&plan);
        for expected in [
            "reverse,setpts=N*1/(30*TB)",
            "setpts=PTS/2",
            "scale=w='iw*2/1':h=ih,setsar=1,transpose=clock",
            "split=2",
            "concat=n=2:v=1:a=0",
            marker,
        ] {
            assert!(graph.contains(expected), "missing {expected}: {graph}");
        }
    }
}

#[test]
fn media_geometry_handles_every_rotation_branch() {
    for (rotation, marker) in [
        (180, "hflip,vflip"),
        (270, "transpose=cclock"),
        (45, "rotate=angle='45*PI/180'"),
    ] {
        let mut plan = resolved(&fixture());
        plan.inputs[0].video.as_mut().unwrap().info.rotation_degrees = rotation;
        let graph = graph(&plan);
        assert!(graph.contains(marker));
        if rotation == 45 {
            assert!(graph.contains(":c=black@0"));
        }
    }
}

#[test]
fn image_and_freeze_sources_emit_loop_and_frame_hold() {
    let mut image = resolved(&fixture());
    image.inputs[0].kind = ResolvedInputKind::Media {
        material_kind: MaterialKind::Image,
    };
    image.inputs[0].audio = None;
    let mapping = clip(&mut image).source_mapping.as_mut().unwrap();
    let ResolvedSourceTimeMap::Linear { direction, .. } = &mut mapping.time_map else {
        panic!("linear mapping");
    };
    *direction = PlaybackDirection::Reverse;
    let image_graph = graph(&image);
    assert!(image_graph.contains("loop=loop=-1:size=1:start=0"));
    assert!(!image_graph.contains("]reverse["));

    let mut freeze = resolved(&fixture());
    let source = freeze.sequences[0].tracks[0].clips[0].source.clone();
    let ResolvedClipSource::Media {
        input_id,
        video_stream: Some(video_stream),
        ..
    } = source
    else {
        unreachable!()
    };
    clip(&mut freeze).source = ResolvedClipSource::FreezeFrame {
        input_id,
        video_stream,
        source_time: time(300),
    };
    let freeze_graph = graph(&freeze);
    assert!(freeze_graph
        .contains("fps=fps=30/1:start_time=0.5:round=down:eof_action=pass,trim=end_frame=1"));
    assert!(freeze_graph.contains("tpad=stop_mode=clone"));

    freeze.inputs[0].kind = ResolvedInputKind::Media {
        material_kind: MaterialKind::Image,
    };
    freeze.inputs[0].audio = None;
    let image_freeze_graph = graph(&freeze);
    assert!(
        image_freeze_graph.contains("loop=loop=-1:size=1:start=0,fps=fps=30/1,trim=end_frame=1")
    );
}

#[test]
fn inconsistent_resolved_media_fails_closed() {
    let mut plan = resolved(&fixture());
    let ResolvedClipSource::Media { video_stream, .. } = &mut clip(&mut plan).source else {
        unreachable!()
    };
    *video_stream = None;
    assert_unsupported(&plan, "PLAN_INPUT_STREAM_INVALID");

    let mut plan = resolved(&fixture());
    plan.inputs[0].kind = ResolvedInputKind::Media {
        material_kind: MaterialKind::Audio,
    };
    assert_unsupported(&plan, "PLAN_INPUT_FACTS_INVALID");

    let mut plan = resolved(&fixture());
    plan.inputs[0].video = None;
    assert_unsupported(&plan, "PLAN_INPUT_FACTS_INVALID");
}

fn clip(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut veac_plan::ResolvedClip {
    &mut plan.sequences[0].tracks[0].clips[0]
}

fn graph(plan: &veac_plan::ResolvedRenderPlan) -> String {
    emit_video_command(plan, &bindings(plan))
        .unwrap()
        .filter_graph
        .unwrap()
}

fn assert_unsupported(plan: &veac_plan::ResolvedRenderPlan, code: &str) {
    let error = emit_video_command(plan, &bindings(plan)).unwrap_err();
    assert_eq!(error.diagnostics()[0].kind, CodegenErrorKind::InvalidPlan);
    assert_eq!(error.diagnostics()[0].code, code);
}
