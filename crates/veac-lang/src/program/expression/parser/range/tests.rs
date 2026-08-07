use super::*;
use crate::program::expression::ast::BinaryOperator;

fn parsed(source: &str) -> Result<Expression, ExpressionError> {
    super::super::parse(crate::program::expression::lexer::lex(source)?)
}

#[test]
fn retains_optional_step_and_complete_span() {
    let plain = parsed("1..10").unwrap();
    assert_eq!(plain.span, 0..5);
    assert!(matches!(
        plain.kind,
        ExpressionKind::Range { step: None, .. }
    ));

    let stepped = parsed("1 .. 10 by 2").unwrap();
    assert_eq!(stepped.span, 0..12);
    assert!(matches!(
        stepped.kind,
        ExpressionKind::Range { step: Some(_), .. }
    ));
}

#[test]
fn additive_binds_tighter_than_range() {
    let expression = parsed("1 + 2 .. 10 - 1 by 2 + 1").unwrap();
    let ExpressionKind::Range { start, end, step } = expression.kind else {
        panic!("expected range");
    };
    assert!(matches!(
        start.kind,
        ExpressionKind::Binary {
            operator: BinaryOperator::Add,
            ..
        }
    ));
    assert!(matches!(
        end.kind,
        ExpressionKind::Binary {
            operator: BinaryOperator::Subtract,
            ..
        }
    ));
    assert!(matches!(
        step.unwrap().kind,
        ExpressionKind::Binary {
            operator: BinaryOperator::Add,
            ..
        }
    ));
}

#[test]
fn range_binds_tighter_than_ordering() {
    for source in ["1 < 2 .. 3", "1 .. 2 < 3"] {
        assert!(matches!(
            parsed(source).unwrap().kind,
            ExpressionKind::Binary {
                operator: BinaryOperator::Less,
                ..
            }
        ));
    }
}

#[test]
fn rejects_a_second_range_separator_at_that_separator() {
    let source = "1 .. 2 .. 3";
    let error = parsed(source).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_RANGE_CHAIN");
    assert_eq!(&source[error.span()], "..");
}

#[test]
fn missing_end_or_step_uses_the_existing_expected_value_error() {
    for source in ["1 ..", "1 .. 2 by"] {
        let error = parsed(source).unwrap_err();
        assert_eq!(error.code(), "EXPRESSION_EXPECTED_VALUE", "{source}");
        assert_eq!(error.span(), source.len()..source.len(), "{source}");
    }
}
