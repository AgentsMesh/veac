use super::*;
use crate::program::expression::hir::TypedNodeKind;
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

#[test]
fn for_accepts_every_closed_iterable_shape() {
    assert_eq!(
        lowered("for value in [1, 2] { value }")
            .unwrap()
            .result_type()
            .to_string(),
        "list<int>"
    );
    assert_eq!(
        lowered("for value in 0 .. 3 { value }")
            .unwrap()
            .result_type()
            .to_string(),
        "list<int>"
    );
    assert_eq!(
        lowered("for entry in #{\"a\": 1} { 2 }")
            .unwrap()
            .result_type()
            .to_string(),
        "list<int>"
    );
}

#[test]
fn for_callback_captures_lexical_values_including_functions() {
    let typed = lowered(
        "{ let add = fn(value: int) -> int effect pure { value + 1 }; for value in [2] { add(value) } }",
    )
    .unwrap();
    let TypedNodeKind::Block(block) = typed.root.kind else {
        panic!("expected block");
    };
    let TypedNodeKind::ForEach { body, .. } = &block.result.kind else {
        panic!("expected for-each");
    };
    let TypedNodeKind::Closure {
        captures,
        non_escaping,
        ..
    } = &body.kind
    else {
        panic!("expected synthesized callback");
    };
    assert!(*non_escaping);
    assert_eq!(captures.len(), 1);
    assert!(matches!(
        captures[0].source.value_type.kind(),
        crate::program::expression::ValueTypeKind::Function { .. }
    ));
}

#[test]
fn for_binding_is_lexical_and_body_receives_expected_type() {
    assert_eq!(
        lowered("for min in [1] { min(2) }").unwrap_err().code(),
        "EXPRESSION_CALL_CALLEE_TYPE"
    );
    lowered("{ let nested: list<list<int>> = for value in [1] { [] }; nested }").unwrap();
}

#[test]
fn for_transports_function_elements_and_results() {
    for source in [
        "for callback in [fn(value: int) -> int effect pure { value }] { 1 }",
        "for value in [1] { fn(next: int) -> int effect pure { next + value } }",
    ] {
        assert!(lowered(source).is_ok(), "{source}");
    }
}
