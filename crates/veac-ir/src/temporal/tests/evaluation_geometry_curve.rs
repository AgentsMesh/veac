use crate::*;

use super::{
    eval_support::evaluate,
    support::{literal, program, scalar},
};

fn curve(left: TemporalValue, right: TemporalValue, result_type: TemporalType) -> TemporalValue {
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
            value_type: result_type,
            kind: TemporalNodeKind::CurveSample {
                input: TemporalNodeId::new(0),
                keys,
            },
            provenance_id: None,
        },
    ];
    evaluate(&program(Vec::new(), nodes, 1, result_type), &[])
}

fn point(x: f64, y: f64) -> TemporalValue {
    TemporalValue::Point {
        value: Point {
            x: Length {
                value: x,
                unit: LengthUnit::Pixels,
            },
            y: Length {
                value: y,
                unit: LengthUnit::Percent,
            },
        },
    }
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> TemporalValue {
    TemporalValue::Rect {
        value: Rect {
            x,
            y,
            width,
            height,
        },
    }
}

#[test]
fn point_and_rect_curves_interpolate_every_leaf() {
    assert_eq!(
        curve(point(0.0, 10.0), point(20.0, 30.0), TemporalType::Point),
        point(10.0, 20.0)
    );
    assert_eq!(
        curve(
            rect(0.0, 0.25, 0.5, 0.75),
            rect(0.5, 0.75, 1.0, 1.25),
            TemporalType::Rect,
        ),
        rect(0.25, 0.5, 0.75, 1.0)
    );
}
