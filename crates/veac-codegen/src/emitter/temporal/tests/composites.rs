use std::sync::Arc;

use veac_plan::canonical::*;

#[path = "composites/values.rs"]
mod values;

use super::support::{assert_code, compile, node, plan, program};
use super::value::{CompiledValue, LengthExpression, LengthKind};
use values::*;

#[test]
fn select_and_equality_cover_every_composite_family() {
    let condition = TemporalValue::Boolean { value: true };
    let families = families();
    let mut nodes = vec![literal(0, condition)];
    for (left, right) in &families {
        let left = push_literal(&mut nodes, left.clone());
        let right = push_literal(&mut nodes, right.clone());
        let id = nodes.len() as u32;
        nodes.push(node(
            id,
            nodes[left as usize].value_type,
            TemporalNodeKind::Select {
                condition: TemporalNodeId::new(0),
                when_true: TemporalNodeId::new(left),
                when_false: TemporalNodeId::new(right),
            },
        ));
    }
    compile_program(nodes, TemporalType::Text).unwrap();

    let mut nodes = Vec::new();
    for (left, right) in families {
        let left = push_literal(&mut nodes, left);
        let right = push_literal(&mut nodes, right);
        let id = nodes.len() as u32;
        nodes.push(node(
            id,
            TemporalType::Boolean,
            TemporalNodeKind::Compare {
                operation: TemporalCompareOperation::Equal,
                left: TemporalNodeId::new(left),
                right: TemporalNodeId::new(right),
            },
        ));
    }
    compile_program(nodes, TemporalType::Boolean).unwrap();
}

#[test]
fn length_unit_choices_and_comparisons_fail_as_typed_backend_limits() {
    for kind in ["select", "compare"] {
        let mut nodes = vec![literal(0, TemporalValue::Boolean { value: true })];
        let left = push_literal(&mut nodes, length(2.0, LengthUnit::Pixels));
        let right = push_literal(&mut nodes, length(0.5, LengthUnit::Normalized));
        let id = nodes.len() as u32;
        let node_kind = if kind == "select" {
            TemporalNodeKind::Select {
                condition: TemporalNodeId::new(0),
                when_true: TemporalNodeId::new(left),
                when_false: TemporalNodeId::new(right),
            }
        } else {
            TemporalNodeKind::Compare {
                operation: TemporalCompareOperation::Equal,
                left: TemporalNodeId::new(left),
                right: TemporalNodeId::new(right),
            }
        };
        let result_type = if kind == "select" {
            TemporalType::Length
        } else {
            TemporalType::Boolean
        };
        nodes.push(node(id, result_type, node_kind));
        assert_code(
            compile_program(nodes, result_type),
            "TEMPORAL_BACKEND_UNSUPPORTED",
        );
    }
}

#[test]
fn compiled_value_sink_accessors_cover_success_and_mismatch_paths() {
    let scalar = CompiledValue::Scalar(expr("1"));
    let angle = CompiledValue::Angle(expr("2"));
    let vector = CompiledValue::Vec2(expr("3"), expr("4"));
    let point = CompiledValue::Point(
        length_expression("5", LengthKind::Pixels),
        length_expression("0.5", LengthKind::Relative),
    );
    let rect = CompiledValue::Rect([expr("1"), expr("2"), expr("3"), expr("4")]);
    assert_eq!(scalar.number_expression().as_deref(), Some("1"));
    assert_eq!(angle.number_expression().as_deref(), Some("2"));
    assert_eq!(vector.vector_expression(0).as_deref(), Some("3"));
    assert_eq!(vector.vector_expression(9).as_deref(), Some("4"));
    assert_eq!(point.point_expression(0, "W").as_deref(), Some("5"));
    assert_eq!(point.point_expression(1, "H").as_deref(), Some("(H)*(0.5)"));
    assert_eq!(rect.rect_expression(3).as_deref(), Some("4"));
    assert!(rect.rect_expression(9).is_none());
    assert!(rect.number_expression().is_none());
    assert!(rect.vector_expression(0).is_none());
    assert!(rect.point_expression(0, "W").is_none());
}

fn compile_program(
    nodes: Vec<TemporalNode>,
    result_type: TemporalType,
) -> Result<CompiledValue, super::TemporalBackendError> {
    let result = nodes.len() as u32 - 1;
    let (plan, binding) = plan(
        program(Vec::new(), nodes, result, result_type),
        Vec::new(),
        Vec::new(),
    );
    compile(&plan, &binding, "t")
}

fn push_literal(nodes: &mut Vec<TemporalNode>, value: TemporalValue) -> u32 {
    let id = nodes.len() as u32;
    nodes.push(literal(id, value));
    id
}

fn literal(id: u32, value: TemporalValue) -> TemporalNode {
    node(id, value.value_type(), TemporalNodeKind::Literal { value })
}

fn expr(value: &str) -> Arc<str> {
    Arc::from(value)
}

fn length_expression(value: &str, kind: LengthKind) -> LengthExpression {
    LengthExpression {
        value: expr(value),
        kind,
    }
}
