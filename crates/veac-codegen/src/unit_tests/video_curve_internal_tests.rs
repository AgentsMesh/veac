use super::*;
use crate::unit_tests::emitter_tests::support::{bindings, fixture, resolved, time};
use veac_plan::canonical::{DeliverableKind, SourceTimeInterpolation, SourceTimeSegment};

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

fn run(segments: &[SourceTimeSegment]) -> Result<String, CodegenErrors> {
    let project = fixture();
    let plan = resolved(&project);
    let execution = bindings(&plan);
    let deliverable = &plan.output.deliverables[0];
    let alpha = match &deliverable.kind {
        DeliverableKind::Video(video) => video.video.alpha,
        _ => panic!("expected video deliverable"),
    };
    let mut context = EmitContext::new_visual(&plan, &execution, deliverable, alpha)?;
    let clip = &plan.sequences[0].tracks[0].clips[0];
    apply(
        &mut context,
        clip,
        Request {
            raw: "0:0",
            segments,
            clock: SourceClock::identity(600).expect("identity clock"),
            image: false,
            info: plan.inputs[0].video.as_ref().map(|stream| &stream.info),
            reverse_facts: None,
        },
    )
}

#[test]
fn emits_linear_and_hold_curve_segments() {
    let mut hold = segment(time(600), time(600));
    hold.interpolation = SourceTimeInterpolation::Hold;
    let label = run(&[segment(time(0), time(600)), hold]).expect("curve filter");
    assert!(!label.is_empty());
}

#[test]
fn rejects_curve_delta_overflow_and_minimum_absolute_value() {
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
        run(&[overflow]).unwrap_err().diagnostics()[0].code,
        "SOURCE_UNSUPPORTED"
    );

    let minimum = segment(
        time(0),
        veac_plan::canonical::RationalTime {
            value: i64::MIN,
            timescale: 600,
        },
    );
    assert_eq!(
        run(&[minimum]).unwrap_err().diagnostics()[0].code,
        "SOURCE_UNSUPPORTED"
    );
}
