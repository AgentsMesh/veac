use crate::*;

use super::support::{codes, literal, program, scalar, seal};

fn curve(keys: Vec<TemporalCurveKey>, output: TemporalType) -> TemporalProgram {
    let nodes = vec![
        literal(0, scalar(0.5)),
        TemporalNode {
            id: TemporalNodeId::new(1),
            value_type: output,
            kind: TemporalNodeKind::CurveSample {
                input: TemporalNodeId::new(0),
                keys,
            },
            provenance_id: None,
        },
    ];
    program(Vec::new(), nodes, 1, output)
}

fn key(position: f64, value: TemporalValue, interpolation: Interpolation) -> TemporalCurveKey {
    TemporalCurveKey {
        position: TemporalCurvePosition::Scalar { value: position },
        value,
        interpolation,
    }
}

fn assert_code(value: &TemporalProgram, expected: &str) {
    let actual = codes(validate_temporal_program(value).unwrap_err());
    assert!(
        actual.iter().any(|code| code == expected),
        "missing {expected}: {actual:?}"
    );
}

#[test]
fn well_typed_ordered_curve_is_valid() {
    validate_temporal_program(&curve(
        vec![
            key(0.0, scalar(0.0), Interpolation::Linear),
            key(1.0, scalar(1.0), Interpolation::Hold),
        ],
        TemporalType::Scalar,
    ))
    .unwrap();
}

#[test]
fn curve_count_position_order_and_driver_types_are_guarded() {
    assert_code(
        &curve(Vec::new(), TemporalType::Scalar),
        "TEMPORAL_CURVE_KEYS",
    );
    let value = curve(
        vec![
            key(1.0, scalar(0.0), Interpolation::Linear),
            key(0.0, scalar(1.0), Interpolation::Hold),
        ],
        TemporalType::Scalar,
    );
    assert_code(&value, "TEMPORAL_CURVE_ORDER");

    let mut value = curve(
        vec![key(0.0, scalar(0.0), Interpolation::Hold)],
        TemporalType::Scalar,
    );
    let TemporalNodeKind::CurveSample { keys, .. } = &mut value.nodes[1].kind else {
        unreachable!()
    };
    keys[0].position = TemporalCurvePosition::Scalar { value: -0.0 };
    assert_code(&value, "TEMPORAL_CURVE_POSITION");

    let TemporalNodeKind::CurveSample { keys, .. } = &mut value.nodes[1].kind else {
        unreachable!()
    };
    keys[0].position = TemporalCurvePosition::Time {
        value: RationalTime::new(0, 30).unwrap(),
    };
    seal(&mut value);
    assert_code(&value, "TEMPORAL_CURVE_TYPE");
}

#[test]
fn curve_values_and_interpolations_are_closed() {
    let value = curve(
        vec![
            key(
                0.0,
                TemporalValue::Boolean { value: false },
                Interpolation::Linear,
            ),
            key(
                1.0,
                TemporalValue::Boolean { value: true },
                Interpolation::Hold,
            ),
        ],
        TemporalType::Boolean,
    );
    assert_code(&value, "TEMPORAL_CURVE_INTERPOLATION");

    let value = curve(
        vec![
            key(
                0.0,
                TemporalValue::Length {
                    value: Length {
                        value: 0.0,
                        unit: LengthUnit::Pixels,
                    },
                },
                Interpolation::Linear,
            ),
            key(
                1.0,
                TemporalValue::Length {
                    value: Length {
                        value: 1.0,
                        unit: LengthUnit::Percent,
                    },
                },
                Interpolation::Hold,
            ),
        ],
        TemporalType::Length,
    );
    assert_code(&value, "TEMPORAL_CURVE_VALUE");

    let value = curve(
        vec![key(
            0.0,
            scalar(0.0),
            Interpolation::CubicBezier {
                x1: 2.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
            },
        )],
        TemporalType::Scalar,
    );
    assert_code(&value, "TEMPORAL_CURVE_TYPE");

    let value = curve(
        vec![key(
            0.0,
            scalar(0.0),
            Interpolation::Spring {
                frequency: 0.0,
                decay: 1.0,
                initial_velocity: 0.0,
            },
        )],
        TemporalType::Scalar,
    );
    assert_code(&value, "TEMPORAL_CURVE_TYPE");
}

#[test]
fn curve_key_limit_is_explicit() {
    let keys = (0..=MAX_TEMPORAL_CURVE_KEYS)
        .map(|index| key(index as f64, scalar(index as f64), Interpolation::Hold))
        .collect();
    assert_code(&curve(keys, TemporalType::Scalar), "TEMPORAL_CURVE_KEYS");
}
