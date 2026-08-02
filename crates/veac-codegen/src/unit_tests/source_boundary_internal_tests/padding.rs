use super::*;
use crate::emitter::EmitContext;
use crate::unit_tests::emitter_tests::multicam::multicam_plan;
use crate::unit_tests::emitter_tests::support::bindings;
use veac_artifact::SourceClock;

#[test]
fn rejects_unbounded_and_unaligned_clocks() {
    let project = fixture();
    let plan = resolved(&project);
    let execution = bindings(&plan);
    let deliverable = &plan.output.deliverables[0];
    let alpha = match &deliverable.kind {
        veac_plan::canonical::DeliverableKind::Video(video) => video.video.alpha,
        _ => panic!("expected video deliverable"),
    };
    let audio_plan = multicam_plan();
    let audio_output = match &audio_plan.output.deliverables[0].kind {
        veac_plan::canonical::DeliverableKind::Video(video) => {
            video.audio.as_ref().expect("audio output")
        }
        _ => panic!("expected video deliverable"),
    };
    let audio_output = AudioRenderSpec::from(audio_output);
    let clip = &plan.sequences[0].tracks[0].clips[0];
    let mut context =
        EmitContext::new_visual(&plan, &execution, deliverable, alpha).expect("context");
    let padding = Padding {
        before: RationalTime::new(1, 44_100).expect("unaligned padding"),
        after: time(0),
    };

    assert_eq!(
        video::pad(
            &mut context,
            "0:0",
            clip,
            SourceClock::identity(600).expect("identity clock"),
            padding,
        )
        .unwrap_err()
        .diagnostics()[0]
            .code,
        "SOURCE_BOUNDARY_POLICY"
    );

    let bounded = SourceClock::bounded(
        TimeRange::new(time(0), time(6_000)).expect("clock"),
        time(0),
    )
    .expect("clock");
    assert_eq!(
        audio::pad(&mut context, "0:1", clip, bounded, padding, &audio_output)
            .unwrap_err()
            .diagnostics()[0]
            .code,
        "SOURCE_BOUNDARY_POLICY"
    );

    let zero = Padding {
        before: time(0),
        after: time(0),
    };
    assert_eq!(
        audio::pad(
            &mut context,
            "0:1",
            clip,
            SourceClock::identity(600).expect("identity clock"),
            zero,
            &audio_output,
        )
        .unwrap_err()
        .diagnostics()[0]
            .code,
        "SOURCE_BOUNDARY_POLICY"
    );

    let after = Padding {
        before: time(0),
        after: RationalTime::new(1, 44_100).expect("unaligned padding"),
    };
    assert_eq!(
        audio::pad(&mut context, "0:1", clip, bounded, after, &audio_output)
            .unwrap_err()
            .diagnostics()[0]
            .code,
        "SOURCE_BOUNDARY_POLICY"
    );
}
