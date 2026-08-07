use veac_plan::canonical::*;

use super::super::super::support::node;

#[path = "cases/point.rs"]
mod point;

pub(super) fn point_case() -> Case {
    point::point_case()
}

pub(super) struct Case {
    pub(super) nodes: Vec<TemporalNode>,
    pub(super) input_type: TemporalType,
    pub(super) clock: TemporalClock,
    pub(super) result: u32,
    pub(super) result_type: TemporalType,
    pub(super) equal: TemporalValue,
}

pub(super) fn vector_case() -> Case {
    let nodes = vec![
        input(0, TemporalType::Scalar),
        literal(1, scalar(2.0)),
        binary(
            2,
            TemporalType::Scalar,
            TemporalBinaryOperation::Multiply,
            0,
            1,
        ),
        node(
            3,
            TemporalType::Vec2,
            TemporalNodeKind::ComposeVec2 {
                x: TemporalNodeId::new(0),
                y: TemporalNodeId::new(2),
            },
        ),
    ];
    case(
        nodes,
        TemporalType::Scalar,
        TemporalClock::Progress,
        TemporalType::Vec2,
        TemporalValue::Vec2 {
            value: Vec2 { x: 0.5, y: 1.0 },
        },
    )
}

pub(super) fn rect_case() -> Case {
    let nodes = vec![
        input(0, TemporalType::Scalar),
        literal(1, scalar(0.1)),
        binary(2, TemporalType::Scalar, TemporalBinaryOperation::Add, 0, 1),
        literal(3, scalar(0.2)),
        literal(4, scalar(0.6)),
        node(
            5,
            TemporalType::Rect,
            TemporalNodeKind::ComposeRect {
                x: TemporalNodeId::new(2),
                y: TemporalNodeId::new(3),
                width: TemporalNodeId::new(4),
                height: TemporalNodeId::new(0),
            },
        ),
    ];
    case(
        nodes,
        TemporalType::Scalar,
        TemporalClock::Progress,
        TemporalType::Rect,
        TemporalValue::Rect {
            value: Rect {
                x: 0.6,
                y: 0.2,
                width: 0.6,
                height: 0.5,
            },
        },
    )
}

pub(super) fn color_case() -> Case {
    let nodes = vec![
        input(0, TemporalType::Integer),
        literal(1, integer(10)),
        binary(2, TemporalType::Integer, TemporalBinaryOperation::Add, 0, 1),
        literal(3, integer(20)),
        literal(4, integer(200)),
        literal(5, integer(255)),
        node(
            6,
            TemporalType::Color,
            TemporalNodeKind::ComposeColor {
                red: TemporalNodeId::new(2),
                green: TemporalNodeId::new(3),
                blue: TemporalNodeId::new(4),
                alpha: TemporalNodeId::new(5),
            },
        ),
    ];
    case(
        nodes,
        TemporalType::Integer,
        TemporalClock::Frame,
        TemporalType::Color,
        TemporalValue::Color {
            value: Color {
                red: 25,
                green: 20,
                blue: 200,
                alpha: 255,
            },
        },
    )
}

pub(super) fn case(
    nodes: Vec<TemporalNode>,
    input_type: TemporalType,
    clock: TemporalClock,
    result_type: TemporalType,
    equal: TemporalValue,
) -> Case {
    Case {
        result: nodes.len() as u32 - 1,
        nodes,
        input_type,
        clock,
        result_type,
        equal,
    }
}

pub(super) fn input(id: u32, value_type: TemporalType) -> TemporalNode {
    node(
        id,
        value_type,
        TemporalNodeKind::Input {
            input_id: TemporalInputId::new(0),
        },
    )
}

pub(super) fn literal(id: u32, value: TemporalValue) -> TemporalNode {
    node(id, value.value_type(), TemporalNodeKind::Literal { value })
}

pub(super) fn binary(
    id: u32,
    value_type: TemporalType,
    operation: TemporalBinaryOperation,
    left: u32,
    right: u32,
) -> TemporalNode {
    node(
        id,
        value_type,
        TemporalNodeKind::Binary {
            operation,
            left: TemporalNodeId::new(left),
            right: TemporalNodeId::new(right),
        },
    )
}

fn scalar(value: f64) -> TemporalValue {
    TemporalValue::Scalar { value }
}

fn integer(value: i64) -> TemporalValue {
    TemporalValue::Integer { value }
}

pub(super) fn length(value: f64) -> TemporalValue {
    TemporalValue::Length {
        value: pixels(value),
    }
}

pub(super) fn pixels(value: f64) -> Length {
    Length {
        value,
        unit: LengthUnit::Pixels,
    }
}
