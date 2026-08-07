use crate::*;

use super::{
    eval_support::evaluate,
    support::{literal, program, scalar},
};

fn scalar_key(at: f64, value: TemporalValue, interpolation: Interpolation) -> TemporalCurveKey {
    TemporalCurveKey {
        position: TemporalCurvePosition::Scalar { value: at },
        value,
        interpolation,
    }
}

fn scalar_curve(sample: f64, keys: Vec<TemporalCurveKey>, result: TemporalType) -> TemporalProgram {
    let nodes = vec![
        literal(0, scalar(sample)),
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
fn scalar_curves_clamp_interpolate_and_hold() {
    let keys = vec![
        scalar_key(0.0, scalar(0.0), Interpolation::Linear),
        scalar_key(1.0, scalar(10.0), Interpolation::Hold),
    ];
    assert_eq!(
        evaluate(&scalar_curve(-1.0, keys.clone(), TemporalType::Scalar), &[]),
        scalar(0.0)
    );
    assert_eq!(
        evaluate(&scalar_curve(0.5, keys.clone(), TemporalType::Scalar), &[]),
        scalar(5.0)
    );
    assert_eq!(
        evaluate(&scalar_curve(2.0, keys, TemporalType::Scalar), &[]),
        scalar(10.0)
    );

    let keys = vec![
        scalar_key(0.0, scalar(2.0), Interpolation::Hold),
        scalar_key(1.0, scalar(9.0), Interpolation::Hold),
    ];
    assert_eq!(
        evaluate(&scalar_curve(0.5, keys, TemporalType::Scalar), &[]),
        scalar(2.0)
    );
}

#[test]
fn time_curves_use_exact_typed_positions() {
    let keys = vec![
        TemporalCurveKey {
            position: TemporalCurvePosition::Time {
                value: RationalTime::new(0, 30).unwrap(),
            },
            value: scalar(0.0),
            interpolation: Interpolation::EaseIn,
        },
        TemporalCurveKey {
            position: TemporalCurvePosition::Time {
                value: RationalTime::new(30, 30).unwrap(),
            },
            value: scalar(8.0),
            interpolation: Interpolation::Hold,
        },
    ];
    let nodes = vec![
        literal(
            0,
            TemporalValue::Time {
                value: RationalTime::new(15, 30).unwrap(),
            },
        ),
        TemporalNode {
            id: TemporalNodeId::new(1),
            value_type: TemporalType::Scalar,
            kind: TemporalNodeKind::CurveSample {
                input: TemporalNodeId::new(0),
                keys,
            },
            provenance_id: None,
        },
    ];
    assert_eq!(
        evaluate(&program(Vec::new(), nodes, 1, TemporalType::Scalar), &[]),
        scalar(2.0)
    );
}

#[test]
fn vector_color_and_text_curves_keep_typed_values() {
    let vectors = vec![
        scalar_key(
            0.0,
            TemporalValue::Vec2 {
                value: Vec2 { x: 0.0, y: 2.0 },
            },
            Interpolation::Linear,
        ),
        scalar_key(
            1.0,
            TemporalValue::Vec2 {
                value: Vec2 { x: 2.0, y: 4.0 },
            },
            Interpolation::Hold,
        ),
    ];
    assert_eq!(
        evaluate(&scalar_curve(0.5, vectors, TemporalType::Vec2), &[]),
        TemporalValue::Vec2 {
            value: Vec2 { x: 1.0, y: 3.0 }
        }
    );
    let colors = vec![
        scalar_key(
            0.0,
            TemporalValue::Color {
                value: Color {
                    red: 0,
                    green: 0,
                    blue: 0,
                    alpha: 0,
                },
            },
            Interpolation::Linear,
        ),
        scalar_key(
            1.0,
            TemporalValue::Color {
                value: Color {
                    red: 255,
                    green: 255,
                    blue: 255,
                    alpha: 255,
                },
            },
            Interpolation::Hold,
        ),
    ];
    assert_eq!(
        evaluate(&scalar_curve(0.5, colors, TemporalType::Color), &[]),
        TemporalValue::Color {
            value: Color {
                red: 128,
                green: 128,
                blue: 128,
                alpha: 128
            }
        }
    );
    let text = vec![
        scalar_key(
            0.0,
            TemporalValue::Text { value: "a".into() },
            Interpolation::Hold,
        ),
        scalar_key(
            1.0,
            TemporalValue::Text { value: "b".into() },
            Interpolation::Hold,
        ),
    ];
    assert_eq!(
        evaluate(&scalar_curve(0.5, text, TemporalType::Text), &[]),
        TemporalValue::Text { value: "a".into() }
    );
}
