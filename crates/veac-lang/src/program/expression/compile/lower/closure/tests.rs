use super::*;
use crate::program::expression::hir::{TypedNode, TypedNodeKind};
use crate::program::expression::{ExpressionContext, PrimitiveType, ValueType};

fn parsed(source: &str) -> Expression {
    crate::program::expression::parser::parse(
        crate::program::expression::lexer::lex(source).unwrap(),
    )
    .unwrap()
}

fn lowered(
    source: &str,
) -> Result<crate::program::expression::hir::TypedExpression, ExpressionError> {
    super::super::expression(&parsed(source), &|_| None, &ExpressionContext::empty())
}

fn closure(
    node: &TypedNode,
) -> (
    &[crate::program::expression::hir::TypedCapture],
    &crate::program::expression::hir::TypedBlock,
) {
    let TypedNodeKind::Closure { captures, body, .. } = &node.kind else {
        panic!("expected closure");
    };
    (captures, body)
}

#[test]
fn authored_closures_are_escaping_values() {
    let typed = lowered("fn(value: int) -> int effect pure { value }").unwrap();
    let TypedNodeKind::Closure { non_escaping, .. } = typed.root.kind else {
        panic!("expected closure");
    };
    assert!(!non_escaping);
}

#[test]
fn closure_type_and_captures_follow_first_resolved_occurrence() {
    let typed = lowered(
        "{ let first = 1; let last = 2; fn(x: int) -> int effect pure { last + x + first } }",
    )
    .unwrap();
    assert_eq!(
        typed.result_type().to_string(),
        "fn(int) -> int effect pure"
    );
    let TypedNodeKind::Block(root) = &typed.root.kind else {
        panic!("expected block");
    };
    let (captures, body) = closure(&root.result);
    assert_eq!(captures.len(), 2);
    assert!(matches!(captures[0].source.kind, TypedNodeKind::Local(_)));
    assert!(matches!(captures[1].source.kind, TypedNodeKind::Local(_)));
    assert_eq!(
        &"{ let first = 1; let last = 2; fn(x: int) -> int effect pure { last + x + first } }"
            [captures[0].source.span.clone()],
        "last"
    );
    assert!(matches!(body.result.kind, TypedNodeKind::Binary { .. }));
}

#[test]
fn nested_closures_bridge_outer_bindings_one_level_at_a_time() {
    let typed = lowered(
        "{ let seed = 1; fn(x: int) -> fn(int) -> int effect pure effect pure { fn(y: int) -> int effect pure { seed + x + y } } }",
    )
    .unwrap();
    let TypedNodeKind::Block(root) = &typed.root.kind else {
        panic!("expected block");
    };
    let (outer_captures, outer_body) = closure(&root.result);
    assert_eq!(outer_captures.len(), 1);
    let (inner_captures, _) = closure(&outer_body.result);
    assert_eq!(inner_captures.len(), 2);
    assert!(matches!(
        inner_captures[0].source.kind,
        TypedNodeKind::Capture(0)
    ));
    assert!(matches!(
        inner_captures[1].source.kind,
        TypedNodeKind::Parameter(0)
    ));
}

#[test]
fn closure_security_rejections_are_specific() {
    let integer = ValueType::primitive(PrimitiveType::Integer);
    let external = super::super::expression(
        &parsed("fn() -> int effect pure { ambient }"),
        &|_| Some(super::super::SymbolTarget::External(integer.clone())),
        &ExpressionContext::empty(),
    )
    .unwrap_err();
    assert_eq!(external.code(), "EXPRESSION_CLOSURE_EXTERNAL_CAPTURE");

    let function = ValueType::function(
        vec![integer.clone()],
        integer,
        crate::program::expression::FunctionEffect::Pure,
    )
    .unwrap();
    let captured = super::super::expression(
        &parsed("fn() -> int effect pure { callback(1) }"),
        &|_| Some(super::super::SymbolTarget::Parameter(0, function.clone())),
        &ExpressionContext::empty(),
    )
    .unwrap_err();
    assert_eq!(captured.code(), "EXPRESSION_CLOSURE_FUNCTION_CAPTURE");
    assert_eq!(
        lowered("{ let recurse = fn() -> int effect pure { recurse() }; recurse }")
            .unwrap_err()
            .code(),
        "EXPRESSION_CLOSURE_SELF_CAPTURE"
    );
}

#[test]
fn value_bindings_shadow_static_calls_and_invoke_types_are_exact() {
    assert_eq!(
        lowered("{ let min = 1; min(2) }").unwrap_err().code(),
        "EXPRESSION_CALL_CALLEE_TYPE"
    );
    assert_eq!(
        lowered("fn(x: int) -> int effect pure { x }()")
            .unwrap_err()
            .code(),
        "EXPRESSION_CALL_ARITY"
    );
    assert_eq!(
        lowered("fn(x: int) -> int effect pure { x }(1s)")
            .unwrap_err()
            .code(),
        "EXPRESSION_CALL_ARGUMENT_TYPE"
    );
    assert_eq!(
        lowered("fn() -> int effect pure { 1s }")
            .unwrap_err()
            .code(),
        "EXPRESSION_CLOSURE_RETURN_TYPE"
    );
}

#[test]
fn only_trusted_external_function_values_enter_typed_hir() {
    let integer = ValueType::primitive(PrimitiveType::Integer);
    let function = ValueType::function(
        vec![integer.clone()],
        integer,
        crate::program::expression::FunctionEffect::Pure,
    )
    .unwrap();
    let typed = super::super::expression(
        &parsed("callback(1)"),
        &|_| {
            Some(super::super::SymbolTarget::TrustedExternal(
                function.clone(),
            ))
        },
        &ExpressionContext::empty(),
    )
    .unwrap();
    let TypedNodeKind::Invoke { callee, .. } = typed.root.kind else {
        panic!("expected dynamic invoke");
    };
    assert!(matches!(callee.kind, TypedNodeKind::External(_)));
}
