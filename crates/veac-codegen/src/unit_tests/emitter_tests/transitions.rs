use veac_codegen::emitter::CodegenErrorKind;
use veac_plan::canonical::*;
use veac_plan::ResolvedClipSource;

use super::support::{
    add_transition, bindings, emit_video_command, fixture, resolved, time, visual,
};

#[test]
fn before_and_after_cut_alignments_materialize_zero_handle_sampling() {
    let before = transition_plan(TransitionAlignment::BeforeCut, TransitionKind::Dissolve);
    let before_graph = graph(&before);
    assert!(before_graph.contains("trim=start=0.8:duration=0.2"));
    assert!(before_graph.contains("trim=end_frame=1,setpts=PTS-STARTPTS"));
    assert!(before_graph.contains("tpad=start_mode=clone:start_duration=0.2"));
    assert!(before_graph.contains("afade=t=out:st=0.8:d=0.2"));
    assert!(!before_graph.contains("afade=t=in"));

    let after = transition_plan(TransitionAlignment::AfterCut, TransitionKind::Dissolve);
    let after_graph = graph(&after);
    assert!(after_graph.contains("trim=start=0.933333333333,reverse,trim=end_frame=1"));
    assert!(after_graph.contains("tpad=stop_mode=clone:stop_duration=0.2"));
    assert!(after_graph.contains("trim=start=0:duration=0.2"));
    assert!(after_graph.contains("afade=t=in:st=0:d=0.2"));
    assert!(!after_graph.contains("afade=t=out"));
}

#[test]
fn typed_transition_parameters_generate_executable_backend_expressions() {
    let cases = [
        (TransitionKind::Dissolve, "xfade=transition=fade"),
        (
            TransitionKind::Fade {
                color: FadeColor::Black,
            },
            "xfade=transition=fadeblack",
        ),
        (
            TransitionKind::Wipe {
                direction: CardinalDirection::Left,
                angle_degrees: 15.0,
                softness: 0.25,
            },
            "195*PI/180",
        ),
        (
            TransitionKind::Slide {
                direction: CardinalDirection::Right,
                amount: 1.5,
            },
            "pow(P\\,1.5)",
        ),
        (
            TransitionKind::Zoom {
                direction: ZoomDirection::Out,
                amount: 2.0,
            },
            "max(1-(pow(P\\,2))",
        ),
        (
            TransitionKind::Circle {
                direction: CircleDirection::Open,
                softness: 0.2,
            },
            "hypot(X-W/2\\,Y-H/2)",
        ),
        (TransitionKind::Pixelize { amount: 0.8 }, "floor(X/"),
    ];
    let mut plan = transition_plan(TransitionAlignment::Centered, TransitionKind::Dissolve);
    for (kind, marker) in cases {
        plan.sequences[0].tracks[0].transitions[0].kind = kind.clone();
        let graph = graph(&plan);
        assert!(graph.contains(marker), "{kind:?} missing {marker}: {graph}");
    }
}

#[test]
fn transition_missing_endpoint_and_visual_are_typed_errors() {
    let mut missing = transition_plan(TransitionAlignment::Centered, TransitionKind::Dissolve);
    missing.sequences[0].tracks[0].transitions[0].outgoing_clip_id =
        ItemId::new("itm_absent").unwrap();
    assert_transition_error(&missing, "outgoing clip is missing");

    let mut missing = transition_plan(TransitionAlignment::Centered, TransitionKind::Dissolve);
    missing.sequences[0].tracks[0].transitions[0].incoming_clip_id =
        ItemId::new("itm_absent").unwrap();
    assert_transition_error(&missing, "incoming clip is missing");

    let mut no_visual = transition_plan(TransitionAlignment::Centered, TransitionKind::Dissolve);
    no_visual.sequences[0].tracks[0].clips[0].visual = None;
    let ResolvedClipSource::Media { video_stream, .. } =
        &mut no_visual.sequences[0].tracks[0].clips[0].source
    else {
        unreachable!()
    };
    *video_stream = None;
    let error = emit_video_command(&no_visual, &bindings(&no_visual)).unwrap_err();
    assert!(
        error
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code == "PLAN_STRUCTURE_INVALID"),
        "error={error}"
    );

    let mut shadowed = transition_plan(TransitionAlignment::Centered, TransitionKind::Dissolve);
    let card = visual().card;
    for clip in &mut shadowed.sequences[0].tracks[0].clips {
        clip.visual.as_mut().unwrap().card = card.clone();
    }
    assert!(graph(&shadowed).contains("shadowv"));
}

pub(crate) fn transition_plan(
    alignment: TransitionAlignment,
    kind: TransitionKind,
) -> veac_plan::ResolvedRenderPlan {
    let mut project = fixture();
    project.project.render_configs[0]
        .video_deliverable_mut()
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
    let track = &mut project.project.sequences[0].tracks[0];
    track.clips[0].audio = Some(audio());
    let transition = Transition {
        kind,
        duration: time(120),
        alignment,
    };
    let mut incoming = track.clips[0].clone();
    incoming.id = ItemId::new("itm_transition_in").unwrap();
    incoming.record_range.start = time(600);
    track.clips.push(incoming);
    add_transition(
        &mut project,
        "seq_main",
        "itm_video",
        "itm_transition_in",
        transition,
    );
    resolved(&project)
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

fn graph(plan: &veac_plan::ResolvedRenderPlan) -> String {
    emit_video_command(plan, &bindings(plan))
        .unwrap()
        .filter_graph
        .unwrap()
}

fn assert_transition_error(plan: &veac_plan::ResolvedRenderPlan, marker: &str) {
    let error = emit_video_command(plan, &bindings(plan)).unwrap_err();
    assert_eq!(error.diagnostics()[0].kind, CodegenErrorKind::InvalidPlan);
    assert!(
        error
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.message.contains(marker)),
        "missing {marker}: {error}"
    );
}
