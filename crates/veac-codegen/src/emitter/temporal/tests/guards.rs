use veac_plan::canonical::*;

#[path = "guards/budgets.rs"]
mod budgets;

use super::support::{assert_code, compile, node, plan, program, scalar};

#[test]
fn unavailable_source_clock_fails_closed_with_a_stable_code() {
    let input_id = TemporalInputId::new(0);
    let mut source_plan = plan(
        input_program(
            input_id,
            TemporalType::Time,
            TemporalInputSource::Clock {
                clock: TemporalClock::SourceTime,
            },
        ),
        vec![(input_id, TemporalClock::SourceTime)],
        Vec::new(),
    );
    source_plan.0.sequences[0].tracks[0].clips[0].source_mapping = None;
    assert_code(
        compile(&source_plan.0, &source_plan.1, "t"),
        "TEMPORAL_SOURCE_CLOCK_UNAVAILABLE",
    );
}

#[test]
fn incompatible_dynamic_length_units_are_a_backend_capability_error() {
    let nodes = vec![
        length_node(0, 10.0, LengthUnit::Pixels),
        length_node(1, 0.5, LengthUnit::Normalized),
        node(
            2,
            TemporalType::Length,
            TemporalNodeKind::Binary {
                operation: TemporalBinaryOperation::Add,
                left: TemporalNodeId::new(0),
                right: TemporalNodeId::new(1),
            },
        ),
    ];
    let (literal_plan, binding) = plan(
        program(Vec::new(), nodes, 2, TemporalType::Length),
        Vec::new(),
        Vec::new(),
    );
    assert_code(
        compile(&literal_plan, &binding, "t"),
        "TEMPORAL_BACKEND_UNSUPPORTED",
    );
}

#[test]
fn missing_binding_and_incomplete_inputs_are_contract_errors() {
    let (literal_plan, binding) = plan(
        program(
            Vec::new(),
            vec![node(
                0,
                TemporalType::Scalar,
                TemporalNodeKind::Literal { value: scalar(1.0) },
            )],
            0,
            TemporalType::Scalar,
        ),
        Vec::new(),
        Vec::new(),
    );
    let missing = TemporalBindingId::new("tbd_missing_backend_test").unwrap();
    assert_code(
        compile(&literal_plan, &missing, "t"),
        "TEMPORAL_BACKEND_CONTRACT",
    );

    let input_id = TemporalInputId::new(0);
    let parameter_id = TemporalParameterId::new("tpm_missing_backend_test").unwrap();
    let (incomplete_plan, _) = plan(
        input_program(
            input_id,
            TemporalType::Scalar,
            TemporalInputSource::Parameter { parameter_id },
        ),
        Vec::new(),
        Vec::new(),
    );
    assert_code(
        compile(&incomplete_plan, &binding, "t"),
        "TEMPORAL_BACKEND_CONTRACT",
    );
}

fn input_program(
    id: TemporalInputId,
    value_type: TemporalType,
    source: TemporalInputSource,
) -> TemporalProgram {
    program(
        vec![TemporalInputDeclaration {
            id,
            value_type,
            source,
        }],
        vec![node(
            0,
            value_type,
            TemporalNodeKind::Input { input_id: id },
        )],
        0,
        value_type,
    )
}

fn length_node(id: u32, value: f64, unit: LengthUnit) -> TemporalNode {
    node(
        id,
        TemporalType::Length,
        TemporalNodeKind::Literal {
            value: TemporalValue::Length {
                value: Length { value, unit },
            },
        },
    )
}
