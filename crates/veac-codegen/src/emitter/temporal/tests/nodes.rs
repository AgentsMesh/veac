use veac_plan::canonical::*;

use super::support::{compile, node, plan, program, scalar};

#[test]
fn compiler_covers_every_closed_temporal_node_variant() {
    let input_id = TemporalInputId::new(0);
    let parameter_id = TemporalParameterId::new("tpm_backend_test").unwrap();
    let inputs = vec![TemporalInputDeclaration {
        id: input_id,
        value_type: TemporalType::Scalar,
        source: TemporalInputSource::Parameter {
            parameter_id: parameter_id.clone(),
        },
    }];
    let length = |value| TemporalValue::Length {
        value: Length {
            value,
            unit: LengthUnit::Pixels,
        },
    };
    let mut nodes = vec![
        node(
            0,
            TemporalType::Scalar,
            TemporalNodeKind::Input { input_id },
        ),
        node(
            1,
            TemporalType::Scalar,
            TemporalNodeKind::Literal { value: scalar(2.0) },
        ),
        node(
            2,
            TemporalType::Scalar,
            TemporalNodeKind::Unary {
                operation: TemporalUnaryOperation::Negate,
                operand: TemporalNodeId::new(0),
            },
        ),
        node(
            3,
            TemporalType::Scalar,
            TemporalNodeKind::Binary {
                operation: TemporalBinaryOperation::Add,
                left: TemporalNodeId::new(1),
                right: TemporalNodeId::new(2),
            },
        ),
        node(
            4,
            TemporalType::Boolean,
            TemporalNodeKind::Compare {
                operation: TemporalCompareOperation::Less,
                left: TemporalNodeId::new(2),
                right: TemporalNodeId::new(1),
            },
        ),
        node(
            5,
            TemporalType::Scalar,
            TemporalNodeKind::Select {
                condition: TemporalNodeId::new(4),
                when_true: TemporalNodeId::new(1),
                when_false: TemporalNodeId::new(0),
            },
        ),
        node(
            6,
            TemporalType::Scalar,
            TemporalNodeKind::CurveSample {
                input: TemporalNodeId::new(0),
                keys: scalar_keys(),
            },
        ),
        node(
            7,
            TemporalType::Vec2,
            TemporalNodeKind::ComposeVec2 {
                x: TemporalNodeId::new(0),
                y: TemporalNodeId::new(1),
            },
        ),
        node(
            8,
            TemporalType::Scalar,
            TemporalNodeKind::ProjectVec2 {
                value: TemporalNodeId::new(7),
                axis: TemporalVectorAxis::Y,
            },
        ),
        node(
            9,
            TemporalType::Length,
            TemporalNodeKind::Literal { value: length(2.0) },
        ),
        node(
            10,
            TemporalType::Length,
            TemporalNodeKind::Literal { value: length(3.0) },
        ),
        node(
            11,
            TemporalType::Point,
            TemporalNodeKind::ComposePoint {
                x: TemporalNodeId::new(9),
                y: TemporalNodeId::new(10),
            },
        ),
        node(
            12,
            TemporalType::Length,
            TemporalNodeKind::ProjectPoint {
                value: TemporalNodeId::new(11),
                axis: TemporalPointAxis::X,
            },
        ),
        node(
            13,
            TemporalType::Rect,
            TemporalNodeKind::ComposeRect {
                x: TemporalNodeId::new(0),
                y: TemporalNodeId::new(1),
                width: TemporalNodeId::new(0),
                height: TemporalNodeId::new(1),
            },
        ),
        node(
            14,
            TemporalType::Scalar,
            TemporalNodeKind::ProjectRect {
                value: TemporalNodeId::new(13),
                field: TemporalRectField::Width,
            },
        ),
    ];
    for value in [1_i64, 2, 3, 255] {
        let id = nodes.len() as u32;
        nodes.push(node(
            id,
            TemporalType::Integer,
            TemporalNodeKind::Literal {
                value: TemporalValue::Integer { value },
            },
        ));
    }
    nodes.push(node(
        19,
        TemporalType::Color,
        TemporalNodeKind::ComposeColor {
            red: TemporalNodeId::new(15),
            green: TemporalNodeId::new(16),
            blue: TemporalNodeId::new(17),
            alpha: TemporalNodeId::new(18),
        },
    ));
    nodes.push(node(
        20,
        TemporalType::Integer,
        TemporalNodeKind::ProjectColor {
            value: TemporalNodeId::new(19),
            channel: TemporalColorChannel::Blue,
        },
    ));
    let (plan, binding) = plan(
        program(inputs, nodes, 14, TemporalType::Scalar),
        Vec::new(),
        vec![(input_id, parameter_id, scalar(0.25))],
    );
    assert_eq!(
        compile(&plan, &binding, "t")
            .unwrap()
            .number_expression()
            .as_deref(),
        Some("0.25")
    );
}

fn scalar_keys() -> Vec<TemporalCurveKey> {
    [0.0, 1.0]
        .into_iter()
        .map(|value| TemporalCurveKey {
            position: TemporalCurvePosition::Scalar { value },
            value: scalar(value),
            interpolation: Interpolation::Linear,
        })
        .collect()
}
