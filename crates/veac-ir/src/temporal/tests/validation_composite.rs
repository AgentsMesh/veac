use crate::*;

use super::support::{codes, literal, program, scalar};

fn assert_code(value: &TemporalProgram, expected: &str) {
    let actual = codes(validate_temporal_program(value).unwrap_err());
    assert!(
        actual.iter().any(|code| code == expected),
        "missing {expected}: {actual:?}"
    );
}

fn node(id: u32, value_type: TemporalType, kind: TemporalNodeKind) -> TemporalNode {
    TemporalNode {
        id: TemporalNodeId::new(id),
        value_type,
        kind,
        provenance_id: None,
    }
}

#[test]
fn composite_nodes_enforce_closed_operand_types() {
    let point = program(
        Vec::new(),
        vec![
            literal(0, scalar(1.0)),
            literal(1, scalar(2.0)),
            node(
                2,
                TemporalType::Point,
                TemporalNodeKind::ComposePoint {
                    x: TemporalNodeId::new(0),
                    y: TemporalNodeId::new(1),
                },
            ),
        ],
        2,
        TemporalType::Point,
    );
    assert_code(&point, "TEMPORAL_OPERATION_TYPE");

    let rect = program(
        Vec::new(),
        vec![
            literal(0, scalar(1.0)),
            literal(1, TemporalValue::Integer { value: 2 }),
            node(
                2,
                TemporalType::Rect,
                TemporalNodeKind::ComposeRect {
                    x: TemporalNodeId::new(0),
                    y: TemporalNodeId::new(0),
                    width: TemporalNodeId::new(1),
                    height: TemporalNodeId::new(0),
                },
            ),
        ],
        2,
        TemporalType::Rect,
    );
    assert_code(&rect, "TEMPORAL_OPERATION_TYPE");

    let projection = program(
        Vec::new(),
        vec![
            literal(0, scalar(1.0)),
            node(
                1,
                TemporalType::Integer,
                TemporalNodeKind::ProjectColor {
                    value: TemporalNodeId::new(0),
                    channel: TemporalColorChannel::Red,
                },
            ),
        ],
        1,
        TemporalType::Integer,
    );
    assert_code(&projection, "TEMPORAL_OPERATION_TYPE");
}

#[test]
fn point_and_rect_values_require_canonical_finite_leaves() {
    for invalid in [
        TemporalValue::Point {
            value: Point {
                x: Length {
                    value: -0.0,
                    unit: LengthUnit::Pixels,
                },
                y: Length {
                    value: 0.0,
                    unit: LengthUnit::Pixels,
                },
            },
        },
        TemporalValue::Rect {
            value: Rect {
                x: 0.0,
                y: f64::INFINITY,
                width: 1.0,
                height: 1.0,
            },
        },
    ] {
        let result_type = invalid.value_type();
        let mut value = program(
            Vec::new(),
            vec![literal(0, scalar(0.0))],
            0,
            TemporalType::Scalar,
        );
        value.result_type = result_type;
        value.nodes[0] = literal(0, invalid);
        assert_code(&value, "TEMPORAL_VALUE");
    }
}

#[test]
fn point_curves_keep_each_axis_unit_stable() {
    let point = |x_unit| TemporalValue::Point {
        value: Point {
            x: Length {
                value: 0.0,
                unit: x_unit,
            },
            y: Length {
                value: 0.0,
                unit: LengthUnit::Pixels,
            },
        },
    };
    let keys = vec![
        TemporalCurveKey {
            position: TemporalCurvePosition::Scalar { value: 0.0 },
            value: point(LengthUnit::Pixels),
            interpolation: Interpolation::Linear,
        },
        TemporalCurveKey {
            position: TemporalCurvePosition::Scalar { value: 1.0 },
            value: point(LengthUnit::Percent),
            interpolation: Interpolation::Hold,
        },
    ];
    let value = program(
        Vec::new(),
        vec![
            literal(0, scalar(0.5)),
            node(
                1,
                TemporalType::Point,
                TemporalNodeKind::CurveSample {
                    input: TemporalNodeId::new(0),
                    keys,
                },
            ),
        ],
        1,
        TemporalType::Point,
    );
    assert_code(&value, "TEMPORAL_CURVE_VALUE");
}
