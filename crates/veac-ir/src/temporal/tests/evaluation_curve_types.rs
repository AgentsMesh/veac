use crate::*;

use super::{
    eval_support::evaluate,
    support::{literal, program, scalar},
};

fn curve(left: TemporalValue, right: TemporalValue, result: TemporalType) -> TemporalProgram {
    let keys = vec![
        TemporalCurveKey {
            position: TemporalCurvePosition::Scalar { value: 0.0 },
            value: left,
            interpolation: Interpolation::Linear,
        },
        TemporalCurveKey {
            position: TemporalCurvePosition::Scalar { value: 1.0 },
            value: right,
            interpolation: Interpolation::Hold,
        },
    ];
    let nodes = vec![
        literal(0, scalar(0.5)),
        TemporalNode {
            id: TemporalNodeId::new(1),
            value_type: result,
            kind: TemporalNodeKind::CurveSample {
                input: TemporalNodeId::new(0),
                keys,
            },
            provenance_id: None,
        },
    ];
    program(Vec::new(), nodes, 1, result)
}

#[test]
fn length_and_angle_curves_preserve_their_types() {
    let length = |value| TemporalValue::Length {
        value: Length {
            value,
            unit: LengthUnit::Pixels,
        },
    };
    assert_eq!(
        evaluate(&curve(length(0.0), length(10.0), TemporalType::Length), &[]),
        length(5.0)
    );
    let angle = |degrees| TemporalValue::Angle { degrees };
    assert_eq!(
        evaluate(&curve(angle(0.0), angle(90.0), TemporalType::Angle), &[]),
        angle(45.0)
    );
}
