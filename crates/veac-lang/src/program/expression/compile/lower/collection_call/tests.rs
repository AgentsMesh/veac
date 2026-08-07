use super::*;
use crate::program::expression::hir::{TypedNode, TypedNodeKind};
use crate::program::expression::ExpressionContext;

fn lowered(
    source: &str,
) -> Result<crate::program::expression::hir::TypedExpression, ExpressionError> {
    let tokens = crate::program::expression::lexer::lex(source)?;
    let expression = crate::program::expression::parser::parse(tokens)?;
    crate::program::expression::compile::lower::expression(
        &expression,
        &|_| None,
        &ExpressionContext::empty(),
    )
}

fn collection_arguments(source: &str) -> Vec<TypedNode> {
    let typed = lowered(source).unwrap();
    let TypedNodeKind::Collection { arguments, .. } = typed.root.kind else {
        panic!("expected collection operation");
    };
    arguments
}

#[test]
fn typed_callbacks_supply_empty_list_element_context() {
    let mapped = collection_arguments("map([], fn(value: int) -> text effect pure { \"x\" })");
    assert_eq!(mapped[0].value_type.to_string(), "list<int>");

    let filtered = collection_arguments("filter([], fn(value: int) -> bool effect pure { true })");
    assert_eq!(filtered[0].value_type.to_string(), "list<int>");
}

#[test]
fn typed_callback_supplies_empty_map_entry_context() {
    let arguments =
        collection_arguments("map(#{}, fn(entry: (text, int)) -> int effect pure { 1 })");
    assert_eq!(arguments[0].value_type.to_string(), "map<text, int>");
}

#[test]
fn fold_callback_types_empty_input_and_accumulator() {
    let arguments = collection_arguments(
        "fold([], [], fn(acc: list<int>, value: int) -> list<int> effect pure { acc })",
    );
    assert_eq!(arguments[0].value_type.to_string(), "list<int>");
    assert_eq!(arguments[1].value_type.to_string(), "list<int>");
}

#[test]
fn aggregate_function_values_are_valid_structural_elements_and_results() {
    for source in [
        "map([fn(value: int) -> int effect pure { value }], fn(value: fn(int) -> int effect pure) -> int effect pure { 1 })",
        "map([1], fn(value: int) -> fn(int) -> int effect pure effect pure { fn(next: int) -> int effect pure { next + value } })",
        "fold([1], fn(value: int) -> int effect pure { value }, fn(acc: fn(int) -> int effect pure, value: int) -> fn(int) -> int effect pure effect pure { acc })",
    ] {
        assert!(lowered(source).is_ok(), "{source}");
    }
}

#[test]
fn collection_calls_reject_non_iterables_and_invalid_callbacks() {
    let cases = [
        "map(1, fn(value: int) -> int effect pure { value })",
        "map([1], 1)",
        "map([1], fn(value: text) -> int effect pure { 1 })",
        "filter([1], fn(value: int) -> int effect pure { value })",
        "fold([1], 0, fn(value: int) -> int effect pure { value })",
        "fold([1], 0, fn(acc: int, value: int) -> text effect pure { \"x\" })",
    ];
    for source in cases {
        assert_eq!(
            lowered(source).unwrap_err().code(),
            "EXPRESSION_CALL_ARGUMENT_TYPE",
            "{source}"
        );
    }
}

#[test]
fn empty_iterable_inference_rejects_unusable_callback_shapes() {
    let cases = [
        "map([], 1)",
        "map([], fn(first: int, second: int) -> int effect pure { first })",
        "map(#{}, fn(value: int) -> int effect pure { value })",
        "map(#{}, fn(value: (text, int, bool)) -> int effect pure { 1 })",
        "map(#{}, fn(value: (scalar, int)) -> int effect pure { 1 })",
        "map(if true { [] } else { [] }, fn(value: int) -> int effect pure { value })",
        "fold([1], [], 1)",
        "fold([1], [], fn(value: int) -> int effect pure { value })",
    ];
    for source in cases {
        assert_eq!(
            lowered(source).unwrap_err().code(),
            "EXPRESSION_CALL_ARGUMENT_TYPE",
            "{source}"
        );
    }
}

#[test]
fn ranges_and_maps_expose_their_collection_element_types() {
    let range = collection_arguments("map(0 .. 2, fn(value: int) -> int effect pure { value })");
    assert_eq!(range[0].value_type.to_string(), "range<int>");
    let map = collection_arguments(
        "filter(#{\"a\": 1}, fn(entry: (text, int)) -> bool effect pure { true })",
    );
    assert_eq!(map[0].value_type.to_string(), "map<text, int>");
}
