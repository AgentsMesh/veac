use super::walk::calls;
use crate::program::expression::hir::TypedExpression;
use crate::program::expression::{ExpressionContext, FunctionDefinition, PrimitiveType, ValueType};

fn integer() -> ValueType {
    PrimitiveType::Integer.into()
}

fn definition(name: &str) -> FunctionDefinition {
    FunctionDefinition::new(name, Vec::new(), integer(), "{ 1 }")
}

fn typed(source: &str, definitions: &[FunctionDefinition]) -> TypedExpression {
    let expression = crate::program::expression::parser::parse(
        crate::program::expression::lexer::lex(source).unwrap(),
    )
    .unwrap();
    let signatures = super::super::signature::declarations(definitions);
    super::super::lower::function_with_signatures(
        &expression,
        &integer(),
        &|_| None,
        &ExpressionContext::empty(),
        &signatures,
    )
    .unwrap()
}

fn ids(definitions: &[FunctionDefinition]) -> Vec<crate::program::expression::FunctionId> {
    definitions
        .iter()
        .map(super::super::function_id::identity)
        .collect()
}

#[test]
fn resolved_calls_inside_range_operands_follow_source_order() {
    let definitions = [
        definition("first"),
        definition("last"),
        definition("stride"),
    ];
    let expression = typed(
        "{ let value = first() .. last() by stride(); 1 }",
        &definitions,
    );
    let targets = calls(&expression)
        .into_iter()
        .map(|call| call.target)
        .collect::<Vec<_>>();
    assert_eq!(targets, ids(&definitions));
}

#[test]
fn dynamic_closure_invokes_are_not_static_edges() {
    let definitions = [definition("helper")];
    let source =
        "{ let callback = fn(value: int) -> int effect pure { value + 1 }; callback(helper()) }";
    let targets = calls(&typed(source, &definitions))
        .into_iter()
        .map(|call| call.target)
        .collect::<Vec<_>>();
    assert_eq!(targets, ids(&definitions));
}

#[test]
fn typed_local_function_values_shadow_static_signatures() {
    let definitions = [definition("helper")];
    let source = "{ let helper = fn(value: int) -> int effect pure { value + 1 }; helper(1) }";
    assert!(calls(&typed(source, &definitions)).is_empty());
}

#[test]
fn iteration_edges_cover_iterable_then_body() {
    let definitions = [definition("values"), definition("nested")];
    let source = "{ let output = for value in values() .. 3 { nested() }; 1 }";
    let targets = calls(&typed(source, &definitions))
        .into_iter()
        .map(|call| call.target)
        .collect::<Vec<_>>();
    assert_eq!(targets, ids(&definitions));
}
