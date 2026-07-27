use super::*;
use crate::unit_tests::emitter_tests::multicam::multicam_plan;
use crate::unit_tests::emitter_tests::support::{bindings, fixture, resolved};
use veac_plan::canonical::{DeliverableKind, RationalTime, SequenceId};
use veac_plan::ResolvedClipSource;

#[test]
fn audio_source_rejects_unaligned_and_unrouted_nested_clips() {
    let project = fixture();
    let plan = resolved(&project);
    let audio_plan = multicam_plan();
    let execution = bindings(&plan);
    let deliverable = &plan.output.deliverables[0];
    let alpha = match &deliverable.kind {
        DeliverableKind::Video(video) => video.video.alpha,
        _ => panic!("expected video deliverable"),
    };
    let output = match &audio_plan.output.deliverables[0].kind {
        DeliverableKind::Video(video) => video.audio.as_ref().expect("audio output"),
        _ => panic!("expected video deliverable"),
    };
    let mut context = EmitContext::new(&plan, &execution, deliverable, alpha).expect("context");
    let base = &plan.sequences[0].tracks[0].clips[0];
    let audio = audio_plan.sequences[0].tracks[0].clips[0].audio.clone();

    let error =
        audio_processing::properties(&mut context, base, "0:1".to_owned(), output).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "AUDIO_PLAN_INVALID");

    let mut unaligned = base.clone();
    unaligned.audio = audio.clone();
    let ResolvedClipSource::Media {
        video_stream,
        audio_stream,
        ..
    } = &mut unaligned.source
    else {
        unreachable!();
    };
    *audio_stream = *video_stream;
    unaligned.record_range.start = RationalTime::new(1, 44_100).expect("unaligned start");
    assert!(audio_source::build(
        &mut context,
        &unaligned,
        output,
        audio_transition_fades::TransitionFades::default(),
        None,
    )
    .is_err());

    let mut missing = base.clone();
    missing.audio = audio.clone();
    missing.source = ResolvedClipSource::Sequence {
        sequence_id: SequenceId::new("seq_missing").expect("sequence id"),
    };
    assert!(audio_source::build(
        &mut context,
        &missing,
        output,
        audio_transition_fades::TransitionFades::default(),
        None,
    )
    .is_err());

    let mut unrouted = base.clone();
    unrouted.audio = audio;
    unrouted.source = ResolvedClipSource::Sequence {
        sequence_id: plan.sequences[0].id.clone(),
    };
    assert!(audio_source::build(
        &mut context,
        &unrouted,
        output,
        audio_transition_fades::TransitionFades::default(),
        None,
    )
    .is_ok());
}
