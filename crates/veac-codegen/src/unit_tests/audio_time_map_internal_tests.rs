use super::*;
use crate::unit_tests::emitter_tests::multicam::multicam_plan;
use crate::unit_tests::emitter_tests::support::bindings;
use veac_plan::canonical::{DeliverableKind, PitchPolicy};

#[test]
fn rejects_an_empty_resolved_audio_curve() {
    let plan = multicam_plan();
    let execution = bindings(&plan);
    let deliverable = &plan.output.deliverables[0];
    let (alpha, output) = match &deliverable.kind {
        DeliverableKind::Video(video) => (
            video.video.alpha,
            video.audio.as_ref().expect("audio output"),
        ),
        _ => panic!("expected video deliverable"),
    };
    let output = AudioRenderSpec::from(output);
    let mut context =
        EmitContext::new_visual(&plan, &execution, deliverable, alpha).expect("context");
    let clip = &plan.sequences[0].tracks[0].clips[0];
    let error = curve(
        &mut context,
        clip,
        CurveRequest {
            raw: "0:1",
            segments: &[],
            clock: SourceClock::identity(600).expect("identity clock"),
            output: &output,
            pitch: PitchPolicy::Preserve,
            reverse_facts: None,
        },
    )
    .unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "AUDIO_PLAN_INVALID");
}
