use veac_plan::canonical::*;

use super::super::support::{node, plan, program};
use super::assert_samples_match;

#[test]
fn time_length_and_angle_scaling_match_reference_samples() {
    for (unit, value_type, clock) in [
        (
            TemporalValue::Time {
                value: RationalTime::new(600, 600).unwrap(),
            },
            TemporalType::Time,
            TemporalClock::ClipTime,
        ),
        (
            TemporalValue::Length {
                value: Length {
                    value: 80.0,
                    unit: LengthUnit::Pixels,
                },
            },
            TemporalType::Length,
            TemporalClock::Progress,
        ),
        (
            TemporalValue::Angle { degrees: 120.0 },
            TemporalType::Angle,
            TemporalClock::Progress,
        ),
    ] {
        for operation in [
            ScaleOperation::UnitTimesClock,
            ScaleOperation::ClockTimesUnit,
        ] {
            let (plan, binding) = scaling_plan(unit.clone(), value_type, clock, operation);
            assert_samples_match(&plan, &binding);
        }
        let (plan, binding) =
            scaling_plan(unit, value_type, clock, ScaleOperation::UnitDividedByTwo);
        assert_samples_match(&plan, &binding);
    }
}

#[derive(Clone, Copy)]
enum ScaleOperation {
    UnitTimesClock,
    ClockTimesUnit,
    UnitDividedByTwo,
}

fn scaling_plan(
    unit: TemporalValue,
    value_type: TemporalType,
    clock: TemporalClock,
    operation: ScaleOperation,
) -> (veac_plan::ResolvedRenderPlan, TemporalBindingId) {
    let input_id = TemporalInputId::new(0);
    let input_type = if value_type == TemporalType::Time {
        TemporalType::Time
    } else {
        TemporalType::Scalar
    };
    let mut nodes = vec![
        node(0, input_type, TemporalNodeKind::Input { input_id }),
        node(1, value_type, TemporalNodeKind::Literal { value: unit }),
        node(
            2,
            TemporalType::Scalar,
            TemporalNodeKind::Literal {
                value: TemporalValue::Scalar { value: 2.0 },
            },
        ),
    ];
    let (binary, left, right) = match operation {
        ScaleOperation::UnitTimesClock if value_type == TemporalType::Time => {
            (TemporalBinaryOperation::Multiply, 0, 2)
        }
        ScaleOperation::ClockTimesUnit if value_type == TemporalType::Time => {
            (TemporalBinaryOperation::Multiply, 2, 0)
        }
        ScaleOperation::UnitDividedByTwo if value_type == TemporalType::Time => {
            (TemporalBinaryOperation::Divide, 0, 2)
        }
        ScaleOperation::UnitTimesClock => (TemporalBinaryOperation::Multiply, 1, 0),
        ScaleOperation::ClockTimesUnit => (TemporalBinaryOperation::Multiply, 0, 1),
        ScaleOperation::UnitDividedByTwo => (TemporalBinaryOperation::Divide, 1, 2),
    };
    nodes.push(node(
        3,
        value_type,
        TemporalNodeKind::Binary {
            operation: binary,
            left: TemporalNodeId::new(left),
            right: TemporalNodeId::new(right),
        },
    ));
    plan(
        program(
            vec![TemporalInputDeclaration {
                id: input_id,
                value_type: input_type,
                source: TemporalInputSource::Clock { clock },
            }],
            nodes,
            3,
            value_type,
        ),
        vec![(input_id, clock)],
        Vec::new(),
    )
}
