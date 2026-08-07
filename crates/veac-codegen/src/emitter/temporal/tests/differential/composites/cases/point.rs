use veac_plan::canonical::*;

use super::super::super::super::support::node;
use super::{binary, case, input, length, literal, pixels, Case};

pub(super) fn point_case() -> Case {
    let nodes = vec![
        input(0, TemporalType::Scalar),
        literal(1, length(40.0)),
        binary(
            2,
            TemporalType::Length,
            TemporalBinaryOperation::Multiply,
            1,
            0,
        ),
        literal(3, length(20.0)),
        binary(
            4,
            TemporalType::Length,
            TemporalBinaryOperation::Multiply,
            3,
            0,
        ),
        node(
            5,
            TemporalType::Point,
            TemporalNodeKind::ComposePoint {
                x: TemporalNodeId::new(2),
                y: TemporalNodeId::new(4),
            },
        ),
    ];
    case(
        nodes,
        TemporalType::Scalar,
        TemporalClock::Progress,
        TemporalType::Point,
        TemporalValue::Point {
            value: Point {
                x: pixels(20.0),
                y: pixels(10.0),
            },
        },
    )
}
