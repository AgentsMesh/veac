use veac_plan::canonical::*;
use veac_plan::ResolvedSourceTimeMap;

#[path = "defense/owners.rs"]
mod owners;
#[path = "defense/reference_clocks.rs"]
mod reference_clocks;

use super::support::{assert_code, node, plan, program, scalar};
use super::{compile_binding, evaluate_binding, TemporalBackendError};
use crate::emitter::process_owner::ProcessOwner;

#[test]
fn malformed_nodes_curves_and_results_fail_without_panicking() {
    let input_id = TemporalInputId::new(0);
    let nodes = vec![node(
        0,
        TemporalType::Scalar,
        TemporalNodeKind::Input { input_id },
    )];
    let (missing_input_plan, binding) = plan(
        program(Vec::new(), nodes, 0, TemporalType::Scalar),
        Vec::new(),
        Vec::new(),
    );
    assert_code(
        compile(&missing_input_plan, &binding),
        "TEMPORAL_BACKEND_CONTRACT",
    );
    assert_eq!(
        evaluate(&missing_input_plan, &binding, 0.0)
            .unwrap_err()
            .code,
        "TEMPORAL_EVALUATION_FAILED"
    );

    let nodes = vec![node(
        0,
        TemporalType::Scalar,
        TemporalNodeKind::Unary {
            operation: TemporalUnaryOperation::Negate,
            operand: TemporalNodeId::new(9),
        },
    )];
    let (plan, binding) = plan(
        program(Vec::new(), nodes, 0, TemporalType::Scalar),
        Vec::new(),
        Vec::new(),
    );
    assert_code(compile(&plan, &binding), "TEMPORAL_BACKEND_CONTRACT");

    let (plan, binding) = literal_plan(9);
    assert_code(compile(&plan, &binding), "TEMPORAL_BACKEND_CONTRACT");
    curve_error(Vec::new());
    curve_error(vec![
        key(0.0, invalid_spring()),
        key(1.0, Interpolation::Hold),
    ]);
}

#[test]
fn source_range_and_reference_evaluator_failures_have_stable_codes() {
    let (mut plan, binding) = clock_plan(TemporalClock::SourceTime);
    let clip = &mut plan.sequences[0].tracks[0].clips[0];
    clip.source_mapping.as_mut().unwrap().time_map = ResolvedSourceTimeMap::Linear {
        source_range_per_repeat: TimeRange {
            start: RationalTime {
                value: MAX_SAFE_INTEGER as i64,
                timescale: 1,
            },
            duration: RationalTime {
                value: 1,
                timescale: 1,
            },
        },
        rate: Rational::new(1, 1).unwrap(),
        repeat: 1,
        direction: PlaybackDirection::Forward,
    };
    assert_code(compile(&plan, &binding), "TEMPORAL_BACKEND_CONTRACT");

    let missing = TemporalBindingId::new("tbd_temporal_absent").unwrap();
    assert_eq!(
        evaluate(&plan, &missing, 0.0).unwrap_err().code,
        "TEMPORAL_BACKEND_CONTRACT"
    );
    let error = TemporalBackendError::new(
        "TEMPORAL_BACKEND_CONTRACT",
        &binding,
        String::from("owned diagnostic"),
    );
    assert_eq!(error.message, "owned diagnostic");
}

fn compile(
    plan: &veac_plan::ResolvedRenderPlan,
    binding: &TemporalBindingId,
) -> Result<super::CompiledValue, TemporalBackendError> {
    let clip = &plan.sequences[0].tracks[0].clips[0];
    compile_binding(plan, binding, ProcessOwner::clip(clip), "t")
}

fn evaluate(
    plan: &veac_plan::ResolvedRenderPlan,
    binding: &TemporalBindingId,
    seconds: f64,
) -> Result<TemporalValue, TemporalBackendError> {
    let clip = &plan.sequences[0].tracks[0].clips[0];
    evaluate_binding(plan, binding, ProcessOwner::clip(clip), seconds)
}

fn literal_plan(result: u32) -> (veac_plan::ResolvedRenderPlan, TemporalBindingId) {
    plan(
        program(
            Vec::new(),
            vec![node(
                0,
                TemporalType::Scalar,
                TemporalNodeKind::Literal { value: scalar(1.0) },
            )],
            result,
            TemporalType::Scalar,
        ),
        Vec::new(),
        Vec::new(),
    )
}

fn curve_error(keys: Vec<TemporalCurveKey>) {
    let nodes = vec![
        node(
            0,
            TemporalType::Scalar,
            TemporalNodeKind::Literal { value: scalar(0.5) },
        ),
        node(
            1,
            TemporalType::Scalar,
            TemporalNodeKind::CurveSample {
                input: TemporalNodeId::new(0),
                keys,
            },
        ),
    ];
    let (plan, binding) = plan(
        program(Vec::new(), nodes, 1, TemporalType::Scalar),
        Vec::new(),
        Vec::new(),
    );
    assert_code(compile(&plan, &binding), "TEMPORAL_BACKEND_CONTRACT");
}

fn key(position: f64, interpolation: Interpolation) -> TemporalCurveKey {
    TemporalCurveKey {
        position: TemporalCurvePosition::Scalar { value: position },
        value: scalar(position),
        interpolation,
    }
}

fn invalid_spring() -> Interpolation {
    Interpolation::Spring {
        frequency: 0.0,
        decay: 1.0,
        initial_velocity: 0.0,
    }
}

fn clock_plan(clock: TemporalClock) -> (veac_plan::ResolvedRenderPlan, TemporalBindingId) {
    let input_id = TemporalInputId::new(0);
    plan(
        program(
            vec![TemporalInputDeclaration {
                id: input_id,
                value_type: clock.value_type(),
                source: TemporalInputSource::Clock { clock },
            }],
            vec![node(
                0,
                clock.value_type(),
                TemporalNodeKind::Input { input_id },
            )],
            0,
            clock.value_type(),
        ),
        vec![(input_id, clock)],
        Vec::new(),
    )
}
