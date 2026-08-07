use veac_plan::canonical::*;

use super::super::support::{node, plan, program, scalar};
use super::{clock_plan, evaluate};

#[test]
fn reference_clock_failures_remain_typed_for_untrusted_plan_data() {
    let (mut missing_mapping, binding) = clock_plan(TemporalClock::SourceTime);
    missing_mapping.sequences[0].tracks[0].clips[0].source_mapping = None;
    let error = evaluate(&missing_mapping, &binding, 0.0).unwrap_err();
    assert_eq!(error.code, "TEMPORAL_SOURCE_CLOCK_UNAVAILABLE");

    let (mut invalid_timebase, binding) = clock_plan(TemporalClock::ClipTime);
    invalid_timebase.sequences[0].duration.timescale = 0;
    let error = evaluate(&invalid_timebase, &binding, 0.0).unwrap_err();
    assert_eq!(error.code, "TEMPORAL_BACKEND_CONTRACT");
    assert!(error.message.contains("rational time contract"));
}

#[test]
fn reference_evaluator_orders_multiple_typed_inputs() {
    let clock_id = TemporalInputId::new(0);
    let parameter_id = TemporalInputId::new(1);
    let parameter = TemporalParameterId::new("tpm_ordered_backend_test").unwrap();
    let inputs = vec![
        TemporalInputDeclaration {
            id: clock_id,
            value_type: TemporalType::Time,
            source: TemporalInputSource::Clock {
                clock: TemporalClock::ClipTime,
            },
        },
        TemporalInputDeclaration {
            id: parameter_id,
            value_type: TemporalType::Scalar,
            source: TemporalInputSource::Parameter {
                parameter_id: parameter.clone(),
            },
        },
    ];
    let (plan, binding) = plan(
        program(
            inputs,
            vec![node(
                0,
                TemporalType::Scalar,
                TemporalNodeKind::Input {
                    input_id: parameter_id,
                },
            )],
            0,
            TemporalType::Scalar,
        ),
        vec![(clock_id, TemporalClock::ClipTime)],
        vec![(parameter_id, parameter, scalar(7.0))],
    );
    assert_eq!(evaluate(&plan, &binding, 0.25).unwrap(), scalar(7.0));
}
