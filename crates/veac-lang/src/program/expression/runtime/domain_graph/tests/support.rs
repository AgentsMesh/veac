use std::sync::Arc;

pub(in crate::program::expression::runtime::domain_graph) use super::*;
use crate::program::expression::{DomainValue, ExecutionBudget};

#[path = "support/descriptions.rs"]
mod descriptions;
#[path = "support/graph.rs"]
mod graph;
#[path = "support/resource.rs"]
mod resource;
#[path = "support/template.rs"]
mod template;
pub(in crate::program::expression::runtime::domain_graph) use descriptions::*;
pub(in crate::program::expression::runtime::domain_graph) use graph::*;
pub(in crate::program::expression::runtime::domain_graph) use resource::*;
pub(in crate::program::expression::runtime::domain_graph) use template::*;

pub(in crate::program::expression::runtime::domain_graph) const DIGEST: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

pub(in crate::program::expression::runtime::domain_graph) fn evaluate(
    graph: &mut DomainGraphTransaction<'_>,
    operation: DomainOperationId,
    operands: Vec<Value>,
) -> Value {
    graph
        .evaluate(operation.opcode(), operands, 10..20)
        .unwrap()
}

pub(in crate::program::expression::runtime::domain_graph) fn identifier(value: &str) -> Value {
    Value::Identifier(Arc::from(value))
}

pub(in crate::program::expression::runtime::domain_graph) fn text(value: &str) -> Value {
    Value::Text(Arc::from(value))
}

pub(in crate::program::expression::runtime::domain_graph) fn length(value: i128) -> Value {
    Value::Length(ExactNumber::integer(value))
}

pub(in crate::program::expression::runtime::domain_graph) fn time(value: i128) -> Value {
    Value::Time(ExactNumber::integer(value))
}

pub(in crate::program::expression::runtime::domain_graph) fn color(value: &str) -> Value {
    Value::Color(Arc::from(value))
}

pub(in crate::program::expression::runtime::domain_graph) fn list(
    kind: DomainType,
    values: Vec<Value>,
) -> Value {
    Value::list(ValueType::domain(kind), values).unwrap()
}

pub(in crate::program::expression::runtime::domain_graph) fn standard(
) -> (DomainOperationRegistry, ExecutionBudget) {
    (DomainOperationRegistry::standard(), Default::default())
}

pub(in crate::program::expression::runtime::domain_graph) fn freeze(
    graph: DomainGraphTransaction<'_>,
    root: &Value,
) -> FrozenDomainGraph {
    graph.freeze(root, 40..50).unwrap()
}

pub(in crate::program::expression::runtime::domain_graph) fn domain(value: &Value) -> &DomainValue {
    let Value::Domain(value) = value else {
        panic!("expected domain handle")
    };
    value
}

pub(in crate::program::expression::runtime::domain_graph) fn primitive(
    kind: PrimitiveType,
    value: i128,
) -> Value {
    Value::from_numeric(kind, ExactNumber::integer(value)).unwrap()
}
