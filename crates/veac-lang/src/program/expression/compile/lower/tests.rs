use super::*;
use crate::program::expression::hir::TypedNodeKind;
use crate::program::expression::FunctionMap;
use crate::program::expression::{
    compile_functions, ExpressionContext, FunctionDefinition, PrimitiveType, ValueType,
};

#[path = "tests/nominal_match.rs"]
mod nominal_match;

fn parsed(source: &str) -> Expression {
    let tokens = crate::program::expression::lexer::lex(source).unwrap();
    crate::program::expression::parser::parse(tokens).unwrap()
}

fn lowered(source: &str) -> Result<TypedExpression, ExpressionError> {
    expression(&parsed(source), &|_| None, &ExpressionContext::empty())
}

fn lowered_function(
    source: &str,
    return_type: &ValueType,
) -> Result<TypedExpression, ExpressionError> {
    function_with_signatures(
        &parsed(source),
        return_type,
        &|_| None,
        &ExpressionContext::empty(),
        &Default::default(),
    )
}

#[test]
fn structural_literals_retain_ordered_typed_hir() {
    let list = lowered("[1, 2]").unwrap();
    assert_eq!(list.result_type().to_string(), "list<int>");
    assert!(matches!(list.root.kind, TypedNodeKind::List(ref values) if values.len() == 2));

    let map = lowered("#{\"b\": 2, \"a\": 1}").unwrap();
    assert_eq!(map.result_type().to_string(), "map<text, int>");
    assert!(matches!(map.root.kind, TypedNodeKind::Map(ref entries) if entries.len() == 2));

    let tuple = lowered("(1, \"x\")").unwrap();
    assert_eq!(tuple.result_type().to_string(), "(int, text)");
    assert!(matches!(tuple.root.kind, TypedNodeKind::Tuple(ref values) if values.len() == 2));
}

#[test]
fn empty_collections_require_and_consume_expected_types() {
    for source in ["[]", "#{}"] {
        assert_eq!(
            lowered(source).unwrap_err().code(),
            "EXPRESSION_COLLECTION_TYPE_CONTEXT"
        );
    }
    let list_type = ValueType::list(PrimitiveType::Integer.into()).unwrap();
    assert_eq!(
        lowered_function("{ [] }", &list_type)
            .unwrap()
            .result_type(),
        &list_type
    );
    assert_eq!(
        lowered("{ let values: list<int> = []; values }")
            .unwrap()
            .result_type(),
        &list_type
    );
}

#[test]
fn peers_supply_collection_context_independently_of_side() {
    for source in ["[] == [1]", "[1] == []"] {
        assert_eq!(
            lowered(source).unwrap().result_type(),
            &ValueType::primitive(PrimitiveType::Boolean)
        );
    }
    for source in ["if true { [] } else { [1] }", "if true { [1] } else { [] }"] {
        assert_eq!(
            lowered(source).unwrap().result_type().to_string(),
            "list<int>"
        );
    }
}

#[test]
fn collection_errors_point_to_the_offending_element() {
    let source = "[1, \"x\"]";
    let error = lowered(source).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_LIST_ELEMENT_TYPE");
    assert_eq!(&source[error.span()], "\"x\"");

    let source = "#{1: \"x\"}";
    let error = lowered(source).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_MAP_KEY_TYPE");
    assert_eq!(&source[error.span()], "1");

    let source = "#{\"a\": 1, \"b\": \"x\"}";
    let error = lowered(source).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_MAP_VALUE_TYPE");
    assert_eq!(&source[error.span()], "\"x\"");
}

#[test]
fn typed_bindings_check_instead_of_casting_values() {
    let source = "{ let values: list<int> = [1.0]; values }";
    let error = lowered(source).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_LIST_ELEMENT_TYPE");
    assert_eq!(&source[error.span()], "1.0");

    let source = "{ let value: int = 1.0; value }";
    let error = lowered(source).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_TYPE");
    assert_eq!(&source[error.span()], "1.0");
}

#[test]
fn value_namespace_head_outranks_an_imported_function_alias() {
    let integer = ValueType::primitive(PrimitiveType::Integer);
    let source = compile_functions(
        &ExpressionContext::empty(),
        &[FunctionDefinition::new(
            "vendor.method",
            Vec::new(),
            integer.clone(),
            "{ 1 }",
        )],
    )
    .unwrap();
    let mut imported = FunctionMap::new();
    imported.bind_from("subject.method", source.functions(), "vendor.method");
    let context = ExpressionContext::empty().with_functions(imported);

    for target in [
        SymbolTarget::Parameter(0, integer.clone()),
        SymbolTarget::External(integer.clone()),
    ] {
        let error = function_with_signatures(
            &parsed("subject.method()"),
            &integer,
            &|name| (name == "subject").then(|| target.clone()),
            &context,
            &Default::default(),
        )
        .unwrap_err();
        assert_eq!(error.code(), "EXPRESSION_METHOD_RECEIVER_TYPE");
    }
}
