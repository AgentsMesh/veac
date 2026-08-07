use std::cell::RefCell;

use super::*;
use crate::program::expression::hir::TypedNodeKind;
use crate::program::expression::{ExpressionContext, PrimitiveType, ValueType};

fn parsed(source: &str) -> Expression {
    let tokens = crate::program::expression::lexer::lex(source).unwrap();
    crate::program::expression::parser::parse(tokens).unwrap()
}

fn lowered(source: &str) -> Result<TypedExpression, ExpressionError> {
    expression(&parsed(source), &|_| None, &ExpressionContext::empty())
}

#[test]
fn range_hir_retains_authored_step_and_semantic_type() {
    let plain = lowered("1 .. 4").unwrap();
    assert_eq!(plain.result_type().to_string(), "range<int>");
    assert!(matches!(
        plain.root.kind,
        TypedNodeKind::Range { step: None, .. }
    ));

    let stepped = lowered("1 .. 4 by 2").unwrap();
    assert!(matches!(
        stepped.root.kind,
        TypedNodeKind::Range { step: Some(_), .. }
    ));
}

#[test]
fn range_rejects_each_non_integer_operand_at_its_own_span() {
    for (source, offending) in [
        ("1.0 .. 4", "1.0"),
        ("1 .. 4s", "4s"),
        ("1 .. 4 by 2.0", "2.0"),
    ] {
        let error = lowered(source).unwrap_err();
        assert_eq!(error.code(), "EXPRESSION_RANGE_TYPE", "{source}");
        assert_eq!(&source[error.span()], offending, "{source}");
    }
}

#[test]
fn range_operands_lower_in_source_order() {
    let seen = RefCell::new(Vec::new());
    let symbols = |name: &str| {
        seen.borrow_mut().push(name.to_owned());
        Some(SymbolTarget::External(ValueType::primitive(
            PrimitiveType::Integer,
        )))
    };
    expression(
        &parsed("first .. last by stride"),
        &symbols,
        &ExpressionContext::empty(),
    )
    .unwrap();
    assert_eq!(&*seen.borrow(), &["first", "last", "stride"]);
}
