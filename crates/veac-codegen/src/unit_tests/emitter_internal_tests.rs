use super::*;
use crate::emitter::audio::AudioRenderSpec;
use crate::unit_tests::emitter_tests::composition_advanced::advanced_plan;
use crate::unit_tests::emitter_tests::multicam::multicam_plan;
use crate::unit_tests::emitter_tests::support::{bindings, fixture, resolved, time};
use veac_plan::canonical::{
    AlphaMode, DeliverableKind, Generator, MulticamAngleId, RationalTime, SequenceId,
};
use veac_plan::{PlanInputId, ResolvedClipSource, ResolvedMulticamSource, ResolvedRenderPlan};

fn alpha(plan: &ResolvedRenderPlan) -> AlphaMode {
    match &plan.output.deliverables[0].kind {
        DeliverableKind::Video(video) => video.video.alpha,
        _ => panic!("expected video deliverable"),
    }
}

fn multicam_source(plan: &ResolvedRenderPlan) -> ResolvedMulticamSource {
    match &plan.sequences[0].tracks[0].clips[0].source {
        ResolvedClipSource::Multicam { source } => source.clone(),
        _ => panic!("expected multicam source"),
    }
}

fn multicam_error(
    plan: &ResolvedRenderPlan,
    source: &ResolvedMulticamSource,
    audio: bool,
) -> String {
    let execution = bindings(plan);
    let deliverable = &plan.output.deliverables[0];
    let mut context =
        EmitContext::new_visual(plan, &execution, deliverable, alpha(plan)).expect("context");
    let clip = &plan.sequences[0].tracks[0].clips[0];
    let result = if audio {
        let output = match &deliverable.kind {
            DeliverableKind::Video(video) => video.audio.as_ref().expect("audio output"),
            _ => unreachable!(),
        };
        let output = AudioRenderSpec::from(output);
        multicam_source::audio(&mut context, clip, source, &output)
    } else {
        multicam_source::video(&mut context, clip, source)
    };
    result.unwrap_err().diagnostics()[0].code.to_owned()
}

#[test]
fn multicam_backend_defends_switch_and_angle_contracts() {
    let plan = multicam_plan();
    let base = multicam_source(&plan);

    let mut source = base.clone();
    source.switches.clear();
    assert_eq!(multicam_error(&plan, &source, false), "SOURCE_UNSUPPORTED");

    let mut source = base.clone();
    source.switches[0].angle_id = MulticamAngleId::new("ang_missing").expect("angle id");
    assert_eq!(multicam_error(&plan, &source, false), "SOURCE_UNSUPPORTED");

    let mut source = base.clone();
    source.angles[0].input_id = PlanInputId::new("pin_missing").expect("input id");
    assert_eq!(multicam_error(&plan, &source, false), "PLAN_INPUT_MISSING");
    assert_eq!(multicam_error(&plan, &source, true), "PLAN_INPUT_MISSING");

    let mut source = base.clone();
    source.angles[0].source_offset = RationalTime {
        value: 9_007_199_254_740_991,
        timescale: 600,
    };
    source.switches[0].range.start = time(1);
    assert_eq!(multicam_error(&plan, &source, false), "SOURCE_UNSUPPORTED");

    let mut source = base;
    source.angles[0].audio_stream = None;
    assert_eq!(multicam_error(&plan, &source, true), "SOURCE_UNSUPPORTED");
}

#[test]
fn source_dispatch_rejects_missing_streams_mappings_and_sequences() {
    let project = fixture();
    let plan = resolved(&project);
    let audio_plan = multicam_plan();
    let execution = bindings(&plan);
    let deliverable = &plan.output.deliverables[0];
    let mut context =
        EmitContext::new_visual(&plan, &execution, deliverable, alpha(&plan)).expect("context");
    let base = &plan.sequences[0].tracks[0].clips[0];

    let mut clip = base.clone();
    let ResolvedClipSource::Media { video_stream, .. } = &mut clip.source else {
        panic!("expected media source");
    };
    *video_stream = None;
    assert_eq!(
        source::video(&mut context, &clip)
            .unwrap_err()
            .diagnostics()[0]
            .code,
        "SOURCE_UNSUPPORTED"
    );

    let mut clip = base.clone();
    clip.audio = audio_plan.sequences[0].tracks[0].clips[0].audio.clone();
    let ResolvedClipSource::Media {
        video_stream,
        audio_stream,
        ..
    } = &mut clip.source
    else {
        unreachable!();
    };
    *audio_stream = *video_stream;
    clip.source_mapping = None;
    let output = AudioRenderSpec::from(context_audio(&audio_plan));
    assert_eq!(
        audio_source::build(
            &mut context,
            &clip,
            &output,
            audio_transition_fades::TransitionFades::default(),
            None,
        )
        .unwrap_err()
        .diagnostics()[0]
            .code,
        "SOURCE_UNSUPPORTED"
    );

    let mut clip = base.clone();
    clip.source = ResolvedClipSource::Sequence {
        sequence_id: SequenceId::new("seq_missing").expect("sequence id"),
    };
    assert_eq!(
        source::video(&mut context, &clip)
            .unwrap_err()
            .diagnostics()[0]
            .code,
        "SOURCE_UNSUPPORTED"
    );

    clip.source = ResolvedClipSource::Generated {
        generator: Generator::Silence,
    };
    assert_eq!(
        source::video(&mut context, &clip)
            .unwrap_err()
            .diagnostics()[0]
            .code,
        "SOURCE_UNSUPPORTED"
    );
}

fn context_audio(plan: &ResolvedRenderPlan) -> &veac_plan::canonical::AudioOutput {
    match &plan.output.deliverables[0].kind {
        DeliverableKind::Video(video) => video.audio.as_ref().expect("audio output"),
        _ => panic!("expected video deliverable"),
    }
}

#[test]
fn process_owner_preserves_clip_and_apply_identity() {
    let plan = advanced_plan();
    let sequence = &plan.sequences[0];
    let clip = &sequence.tracks[0].clips[0];
    let clip_owner = process_owner::ProcessOwner::clip(clip);
    assert_eq!(clip_owner.id(), clip.id.as_str());
    assert_eq!(clip_owner.kind(), "clip");
    assert_eq!(clip_owner.as_clip().expect("clip owner").id, clip.id);

    let apply = &sequence.applies[0];
    let apply_owner = process_owner::ProcessOwner::apply(apply);
    assert_eq!(apply_owner.id(), apply.id.as_str());
    assert_eq!(apply_owner.kind(), "apply");
    assert!(apply_owner.as_clip().is_none());
}

#[path = "emitter_internal_coverage_tests.rs"]
mod coverage_tests;
