use veac_plan::canonical::*;

use super::support::{compile, node, plan, program, scalar};

#[test]
fn compiler_interpolates_every_temporal_curve_result_family() {
    let driver = node(
        0,
        TemporalType::Scalar,
        TemporalNodeKind::Literal { value: scalar(0.5) },
    );
    let values = [
        (TemporalType::Scalar, scalar(0.0), scalar(1.0)),
        (TemporalType::Length, length(0.0), length(1.0)),
        (TemporalType::Angle, angle(0.0), angle(90.0)),
        (TemporalType::Vec2, vector(0.0), vector(1.0)),
        (TemporalType::Point, point(0.0), point(1.0)),
        (TemporalType::Rect, rect(0.0), rect(1.0)),
        (TemporalType::Color, color(0), color(255)),
    ];
    let mut nodes = vec![driver];
    for (value_type, left, right) in values {
        let id = nodes.len() as u32;
        nodes.push(node(
            id,
            value_type,
            TemporalNodeKind::CurveSample {
                input: TemporalNodeId::new(0),
                keys: keys(left, right),
            },
        ));
    }
    let (plan, binding) = plan(
        program(Vec::new(), nodes, 7, TemporalType::Color),
        Vec::new(),
        Vec::new(),
    );
    assert!(matches!(
        compile(&plan, &binding, "t").unwrap(),
        super::super::CompiledValue::Color(_)
    ));
}

#[test]
fn time_driver_and_all_easing_families_compile() {
    let mut nodes = vec![node(
        0,
        TemporalType::Time,
        TemporalNodeKind::Literal {
            value: TemporalValue::Time { value: time(300) },
        },
    )];
    let easings = [
        Interpolation::Hold,
        Interpolation::Linear,
        Interpolation::EaseIn,
        Interpolation::EaseOut,
        Interpolation::EaseInOut,
        Interpolation::Spring {
            frequency: 1.0,
            decay: 4.0,
            initial_velocity: 0.0,
        },
        Interpolation::CubicBezier {
            x1: 0.2,
            y1: 0.1,
            x2: 0.8,
            y2: 0.9,
        },
    ];
    for interpolation in easings {
        let id = nodes.len() as u32;
        nodes.push(node(
            id,
            TemporalType::Scalar,
            TemporalNodeKind::CurveSample {
                input: TemporalNodeId::new(0),
                keys: vec![
                    time_key(0, 0.0, interpolation),
                    time_key(600, 1.0, Interpolation::Hold),
                ],
            },
        ));
    }
    let (plan, binding) = plan(
        program(Vec::new(), nodes, 7, TemporalType::Scalar),
        Vec::new(),
        Vec::new(),
    );
    let expression = compile(&plan, &binding, "t")
        .unwrap()
        .number_expression()
        .unwrap();
    assert!(expression.contains("root("), "{expression}");
}

fn keys(left: TemporalValue, right: TemporalValue) -> Vec<TemporalCurveKey> {
    vec![
        TemporalCurveKey {
            position: TemporalCurvePosition::Scalar { value: 0.0 },
            value: left,
            interpolation: Interpolation::EaseInOut,
        },
        TemporalCurveKey {
            position: TemporalCurvePosition::Scalar { value: 1.0 },
            value: right,
            interpolation: Interpolation::Hold,
        },
    ]
}

fn time_key(ticks: i64, value: f64, interpolation: Interpolation) -> TemporalCurveKey {
    TemporalCurveKey {
        position: TemporalCurvePosition::Time { value: time(ticks) },
        value: scalar(value),
        interpolation,
    }
}

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 600).unwrap()
}
fn length(value: f64) -> TemporalValue {
    TemporalValue::Length {
        value: Length {
            value,
            unit: LengthUnit::Pixels,
        },
    }
}
fn angle(degrees: f64) -> TemporalValue {
    TemporalValue::Angle { degrees }
}
fn vector(value: f64) -> TemporalValue {
    TemporalValue::Vec2 {
        value: Vec2 { x: value, y: value },
    }
}
fn point(value: f64) -> TemporalValue {
    TemporalValue::Point {
        value: Point {
            x: Length {
                value,
                unit: LengthUnit::Normalized,
            },
            y: Length {
                value,
                unit: LengthUnit::Normalized,
            },
        },
    }
}
fn rect(value: f64) -> TemporalValue {
    TemporalValue::Rect {
        value: Rect {
            x: value,
            y: value,
            width: value,
            height: value,
        },
    }
}
fn color(value: u8) -> TemporalValue {
    TemporalValue::Color {
        value: Color {
            red: value,
            green: value,
            blue: value,
            alpha: value,
        },
    }
}
