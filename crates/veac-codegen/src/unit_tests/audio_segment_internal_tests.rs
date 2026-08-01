use super::*;
use crate::unit_tests::emitter_tests::multicam::multicam_plan;
use crate::unit_tests::emitter_tests::support::{bindings, time};
use veac_plan::canonical::{DeliverableKind, PitchPolicy, SourceTimeInterpolation};

fn segment(
    start: veac_plan::canonical::RationalTime,
    end: veac_plan::canonical::RationalTime,
) -> SourceTimeSegment {
    SourceTimeSegment {
        record_duration: time(600),
        source_start: start,
        source_end: end,
        interpolation: SourceTimeInterpolation::Linear,
    }
}

fn run(segment: SourceTimeSegment) -> Result<String, CodegenErrors> {
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
    let mut context = EmitContext::new_visual(&plan, &execution, deliverable, alpha)?;
    let clip = &plan.sequences[0].tracks[0].clips[0];
    filter(
        &mut context,
        clip,
        "0:1",
        &segment,
        SourceClock::identity(600).expect("identity clock"),
        &output,
        PitchPolicy::Preserve,
    )
}

#[test]
fn emits_linear_and_hold_audio_segments() {
    assert!(!run(segment(time(0), time(600)))
        .expect("linear segment")
        .is_empty());
    let mut hold = segment(time(600), time(600));
    hold.interpolation = SourceTimeInterpolation::Hold;
    assert!(!run(hold).expect("hold segment").is_empty());
}

#[test]
fn rejects_audio_delta_overflow_and_minimum_absolute_value() {
    let overflow = segment(
        veac_plan::canonical::RationalTime {
            value: i64::MIN,
            timescale: 600,
        },
        veac_plan::canonical::RationalTime {
            value: i64::MAX,
            timescale: 600,
        },
    );
    assert_eq!(
        run(overflow).unwrap_err().diagnostics()[0].code,
        "AUDIO_PLAN_INVALID"
    );

    let minimum = segment(
        time(0),
        veac_plan::canonical::RationalTime {
            value: i64::MIN,
            timescale: 600,
        },
    );
    assert_eq!(
        run(minimum).unwrap_err().diagnostics()[0].code,
        "AUDIO_PLAN_INVALID"
    );
}
