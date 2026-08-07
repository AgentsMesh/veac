use super::super::support::*;
use super::temporal_node;

pub(super) fn scaled_progress(scale: f64) -> Vec<TemporalNode> {
    vec![
        temporal_node(
            0,
            TemporalType::Scalar,
            TemporalNodeKind::Input {
                input_id: TemporalInputId::new(0),
            },
        ),
        temporal_node(
            1,
            TemporalType::Scalar,
            TemporalNodeKind::Literal {
                value: TemporalValue::Scalar { value: scale },
            },
        ),
        temporal_node(
            2,
            TemporalType::Scalar,
            TemporalNodeKind::Binary {
                operation: TemporalBinaryOperation::Multiply,
                left: TemporalNodeId::new(0),
                right: TemporalNodeId::new(1),
            },
        ),
    ]
}

pub(super) fn progress_curve(start: TemporalValue, end: TemporalValue) -> Vec<TemporalNode> {
    curve(
        TemporalType::Scalar,
        TemporalCurvePosition::Scalar { value: 0.0 },
        TemporalCurvePosition::Scalar { value: 1.0 },
        start,
        end,
    )
}

pub(super) fn sequence_curve(start: TemporalValue, end: TemporalValue) -> Vec<TemporalNode> {
    curve(
        TemporalType::Time,
        TemporalCurvePosition::Time { value: time(0) },
        TemporalCurvePosition::Time { value: time(1_000) },
        start,
        end,
    )
}

fn curve(
    input_type: TemporalType,
    first: TemporalCurvePosition,
    second: TemporalCurvePosition,
    start: TemporalValue,
    end: TemporalValue,
) -> Vec<TemporalNode> {
    let result_type = start.value_type();
    vec![
        temporal_node(
            0,
            input_type,
            TemporalNodeKind::Input {
                input_id: TemporalInputId::new(0),
            },
        ),
        temporal_node(
            1,
            result_type,
            TemporalNodeKind::CurveSample {
                input: TemporalNodeId::new(0),
                keys: vec![key(first, start), key(second, end)],
            },
        ),
    ]
}

fn key(position: TemporalCurvePosition, value: TemporalValue) -> TemporalCurveKey {
    TemporalCurveKey {
        position,
        value,
        interpolation: Interpolation::Linear,
    }
}
