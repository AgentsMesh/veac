use crate::*;

use super::{
    eval_support::evaluate,
    support::{literal, program, scalar},
};

fn length(value: f64, unit: LengthUnit) -> TemporalValue {
    TemporalValue::Length {
        value: Length { value, unit },
    }
}

fn integer(value: i64) -> TemporalValue {
    TemporalValue::Integer { value }
}

fn run(nodes: Vec<TemporalNode>, result: u32, result_type: TemporalType) -> TemporalValue {
    evaluate(&program(Vec::new(), nodes, result, result_type), &[])
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
fn point_construction_and_projection_preserve_length_units() {
    let base = vec![
        literal(0, length(10.0, LengthUnit::Pixels)),
        literal(1, length(25.0, LengthUnit::Percent)),
        node(
            2,
            TemporalType::Point,
            TemporalNodeKind::ComposePoint {
                x: TemporalNodeId::new(0),
                y: TemporalNodeId::new(1),
            },
        ),
    ];
    for (axis, expected) in [
        (TemporalPointAxis::X, length(10.0, LengthUnit::Pixels)),
        (TemporalPointAxis::Y, length(25.0, LengthUnit::Percent)),
    ] {
        let mut nodes = base.clone();
        nodes.push(node(
            3,
            TemporalType::Length,
            TemporalNodeKind::ProjectPoint {
                value: TemporalNodeId::new(2),
                axis,
            },
        ));
        assert_eq!(run(nodes, 3, TemporalType::Length), expected);
    }
}

#[test]
fn rectangle_construction_projects_every_closed_field() {
    let mut base = vec![
        literal(0, scalar(1.0)),
        literal(1, scalar(2.0)),
        literal(2, scalar(3.0)),
        literal(3, scalar(4.0)),
    ];
    base.push(node(
        4,
        TemporalType::Rect,
        TemporalNodeKind::ComposeRect {
            x: TemporalNodeId::new(0),
            y: TemporalNodeId::new(1),
            width: TemporalNodeId::new(2),
            height: TemporalNodeId::new(3),
        },
    ));
    for (field, expected) in [
        (TemporalRectField::X, 1.0),
        (TemporalRectField::Y, 2.0),
        (TemporalRectField::Width, 3.0),
        (TemporalRectField::Height, 4.0),
    ] {
        let mut nodes = base.clone();
        nodes.push(node(
            5,
            TemporalType::Scalar,
            TemporalNodeKind::ProjectRect {
                value: TemporalNodeId::new(4),
                field,
            },
        ));
        assert_eq!(run(nodes, 5, TemporalType::Scalar), scalar(expected));
    }
}

#[test]
fn color_construction_projects_every_rgba_channel() {
    let mut base = vec![
        literal(0, integer(10)),
        literal(1, integer(20)),
        literal(2, integer(30)),
        literal(3, integer(40)),
    ];
    base.push(node(
        4,
        TemporalType::Color,
        TemporalNodeKind::ComposeColor {
            red: TemporalNodeId::new(0),
            green: TemporalNodeId::new(1),
            blue: TemporalNodeId::new(2),
            alpha: TemporalNodeId::new(3),
        },
    ));
    for (channel, expected) in [
        (TemporalColorChannel::Red, 10),
        (TemporalColorChannel::Green, 20),
        (TemporalColorChannel::Blue, 30),
        (TemporalColorChannel::Alpha, 40),
    ] {
        let mut nodes = base.clone();
        nodes.push(node(
            5,
            TemporalType::Integer,
            TemporalNodeKind::ProjectColor {
                value: TemporalNodeId::new(4),
                channel,
            },
        ));
        assert_eq!(run(nodes, 5, TemporalType::Integer), integer(expected));
    }
}

#[test]
fn color_channels_are_checked_at_runtime() {
    let nodes = vec![
        literal(0, integer(256)),
        literal(1, integer(0)),
        node(
            2,
            TemporalType::Color,
            TemporalNodeKind::ComposeColor {
                red: TemporalNodeId::new(0),
                green: TemporalNodeId::new(1),
                blue: TemporalNodeId::new(1),
                alpha: TemporalNodeId::new(1),
            },
        ),
    ];
    let error = evaluate_temporal_program(
        &program(Vec::new(), nodes, 2, TemporalType::Color),
        &[],
        TemporalEvaluationLimits::default(),
    )
    .unwrap_err();
    assert_eq!(error.code(), "TEMPORAL_EVALUATION_VALUE");
}
