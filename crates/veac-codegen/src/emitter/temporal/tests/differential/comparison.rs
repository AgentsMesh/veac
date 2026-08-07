use veac_plan::canonical::*;

use super::super::support::{node, plan, program};
use super::assert_samples_match;

#[test]
fn every_comparison_operator_matches_scalar_reference_samples() {
    for operation in operations() {
        let (plan, binding) = direct_plan(
            TemporalType::Scalar,
            TemporalClock::Progress,
            TemporalValue::Scalar { value: 0.5 },
            operation,
        );
        assert_samples_match(&plan, &binding);
    }
}

#[test]
fn dimensioned_and_integer_comparisons_match_reference_samples() {
    for (value_type, clock, threshold) in [
        (
            TemporalType::Integer,
            TemporalClock::Frame,
            TemporalValue::Integer { value: 15 },
        ),
        (
            TemporalType::Time,
            TemporalClock::ClipTime,
            TemporalValue::Time {
                value: RationalTime::new(300, 600).unwrap(),
            },
        ),
    ] {
        for operation in operations() {
            let (plan, binding) = direct_plan(value_type, clock, threshold.clone(), operation);
            assert_samples_match(&plan, &binding);
        }
    }
    for (unit, threshold) in [
        (
            TemporalValue::Length {
                value: Length {
                    value: 100.0,
                    unit: LengthUnit::Pixels,
                },
            },
            TemporalValue::Length {
                value: Length {
                    value: 50.0,
                    unit: LengthUnit::Pixels,
                },
            },
        ),
        (
            TemporalValue::Angle { degrees: 100.0 },
            TemporalValue::Angle { degrees: 50.0 },
        ),
    ] {
        for operation in operations() {
            let (plan, binding) = scaled_plan(unit.clone(), threshold.clone(), operation);
            assert_samples_match(&plan, &binding);
        }
    }
}

fn direct_plan(
    value_type: TemporalType,
    clock: TemporalClock,
    threshold: TemporalValue,
    operation: TemporalCompareOperation,
) -> (veac_plan::ResolvedRenderPlan, TemporalBindingId) {
    let input_id = TemporalInputId::new(0);
    let nodes = vec![
        node(0, value_type, TemporalNodeKind::Input { input_id }),
        literal(1, threshold),
        compare(2, operation, 0, 1),
    ];
    plan(
        program(
            vec![clock_input(input_id, value_type, clock)],
            nodes,
            2,
            TemporalType::Boolean,
        ),
        vec![(input_id, clock)],
        Vec::new(),
    )
}

fn scaled_plan(
    unit: TemporalValue,
    threshold: TemporalValue,
    operation: TemporalCompareOperation,
) -> (veac_plan::ResolvedRenderPlan, TemporalBindingId) {
    let input_id = TemporalInputId::new(0);
    let unit_type = unit.value_type();
    let nodes = vec![
        node(
            0,
            TemporalType::Scalar,
            TemporalNodeKind::Input { input_id },
        ),
        literal(1, unit),
        node(
            2,
            unit_type,
            TemporalNodeKind::Binary {
                operation: TemporalBinaryOperation::Multiply,
                left: TemporalNodeId::new(1),
                right: TemporalNodeId::new(0),
            },
        ),
        literal(3, threshold),
        compare(4, operation, 2, 3),
    ];
    plan(
        program(
            vec![clock_input(
                input_id,
                TemporalType::Scalar,
                TemporalClock::Progress,
            )],
            nodes,
            4,
            TemporalType::Boolean,
        ),
        vec![(input_id, TemporalClock::Progress)],
        Vec::new(),
    )
}

fn compare(id: u32, operation: TemporalCompareOperation, left: u32, right: u32) -> TemporalNode {
    node(
        id,
        TemporalType::Boolean,
        TemporalNodeKind::Compare {
            operation,
            left: TemporalNodeId::new(left),
            right: TemporalNodeId::new(right),
        },
    )
}

fn literal(id: u32, value: TemporalValue) -> TemporalNode {
    node(id, value.value_type(), TemporalNodeKind::Literal { value })
}

fn clock_input(
    id: TemporalInputId,
    value_type: TemporalType,
    clock: TemporalClock,
) -> TemporalInputDeclaration {
    TemporalInputDeclaration {
        id,
        value_type,
        source: TemporalInputSource::Clock { clock },
    }
}

fn operations() -> [TemporalCompareOperation; 6] {
    use TemporalCompareOperation::*;
    [Equal, NotEqual, Less, LessOrEqual, Greater, GreaterOrEqual]
}
