use crate::*;

use super::{
    eval_support::{compare, evaluate},
    support::{literal, program, scalar},
};

#[test]
fn comparisons_cover_equality_ordering_and_exact_time() {
    use TemporalCompareOperation as Op;
    assert_eq!(
        compare(Op::Equal, scalar(2.0), scalar(2.0)),
        TemporalValue::Boolean { value: true }
    );
    assert_eq!(
        compare(Op::NotEqual, scalar(2.0), scalar(3.0)),
        TemporalValue::Boolean { value: true }
    );
    assert_eq!(
        compare(Op::Less, scalar(2.0), scalar(3.0)),
        TemporalValue::Boolean { value: true }
    );
    assert_eq!(
        compare(Op::LessOrEqual, scalar(2.0), scalar(2.0)),
        TemporalValue::Boolean { value: true }
    );
    assert_eq!(
        compare(Op::Greater, scalar(3.0), scalar(2.0)),
        TemporalValue::Boolean { value: true }
    );
    assert_eq!(
        compare(Op::GreaterOrEqual, scalar(3.0), scalar(3.0)),
        TemporalValue::Boolean { value: true }
    );
    assert_eq!(
        compare(
            Op::Equal,
            TemporalValue::Time {
                value: RationalTime::new(1, 2).unwrap()
            },
            TemporalValue::Time {
                value: RationalTime::new(2, 4).unwrap()
            },
        ),
        TemporalValue::Boolean { value: true }
    );
}

#[test]
fn equality_is_defined_for_every_closed_value_type() {
    use TemporalCompareOperation::Equal;
    let values = vec![
        TemporalValue::Boolean { value: true },
        TemporalValue::Integer { value: 2 },
        TemporalValue::Angle { degrees: 2.0 },
        TemporalValue::Vec2 {
            value: Vec2 { x: 1.0, y: 2.0 },
        },
        TemporalValue::Color {
            value: Color {
                red: 1,
                green: 2,
                blue: 3,
                alpha: 4,
            },
        },
        TemporalValue::Text {
            value: "title".to_owned(),
        },
    ];
    for value in values {
        assert_eq!(
            compare(Equal, value.clone(), value),
            TemporalValue::Boolean { value: true }
        );
    }
    assert_eq!(
        compare(
            Equal,
            TemporalValue::Length {
                value: Length {
                    value: 1.0,
                    unit: LengthUnit::Normalized
                }
            },
            TemporalValue::Length {
                value: Length {
                    value: 100.0,
                    unit: LengthUnit::Percent
                }
            },
        ),
        TemporalValue::Boolean { value: true }
    );
}

#[test]
fn select_evaluates_a_typed_pure_value() {
    for (condition, expected) in [(true, 10.0), (false, 20.0)] {
        let nodes = vec![
            literal(0, TemporalValue::Boolean { value: condition }),
            literal(1, scalar(10.0)),
            literal(2, scalar(20.0)),
            TemporalNode {
                id: TemporalNodeId::new(3),
                value_type: TemporalType::Scalar,
                kind: TemporalNodeKind::Select {
                    condition: TemporalNodeId::new(0),
                    when_true: TemporalNodeId::new(1),
                    when_false: TemporalNodeId::new(2),
                },
                provenance_id: None,
            },
        ];
        assert_eq!(
            evaluate(&program(Vec::new(), nodes, 3, TemporalType::Scalar), &[]),
            scalar(expected)
        );
    }
}

#[test]
fn vector_composition_and_projection_are_explicit_nodes() {
    for (axis, expected) in [(TemporalVectorAxis::X, 3.0), (TemporalVectorAxis::Y, 4.0)] {
        let nodes = vec![
            literal(0, scalar(3.0)),
            literal(1, scalar(4.0)),
            TemporalNode {
                id: TemporalNodeId::new(2),
                value_type: TemporalType::Vec2,
                kind: TemporalNodeKind::ComposeVec2 {
                    x: TemporalNodeId::new(0),
                    y: TemporalNodeId::new(1),
                },
                provenance_id: None,
            },
            TemporalNode {
                id: TemporalNodeId::new(3),
                value_type: TemporalType::Scalar,
                kind: TemporalNodeKind::ProjectVec2 {
                    value: TemporalNodeId::new(2),
                    axis,
                },
                provenance_id: None,
            },
        ];
        assert_eq!(
            evaluate(&program(Vec::new(), nodes, 3, TemporalType::Scalar), &[]),
            scalar(expected)
        );
    }
}
