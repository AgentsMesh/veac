use super::*;
use crate::program::expression::ast::ExpressionKind;

fn parsed(source: &str) -> Result<Expression, ExpressionError> {
    super::super::parse(crate::program::expression::lexer::lex(source)?)
}

#[test]
fn match_retains_variant_shorthand_renamed_and_wildcard_patterns() {
    let source = "match state { State.First => first, \
                  State.Second { field, other: renamed, } => field, _ => fallback, }";
    let expression = parsed(source).unwrap();
    assert_eq!(expression.span, 0..source.len());
    let ExpressionKind::Match { scrutinee, arms } = expression.kind else {
        panic!("expected match");
    };
    assert!(matches!(scrutinee.kind, ExpressionKind::Symbol(_)));
    assert_eq!(arms.len(), 3);
    let MatchPattern::Variant { path, fields } = &arms[1].pattern else {
        panic!("expected payload pattern");
    };
    assert_eq!(path.last().unwrap().name, "Second");
    assert_eq!(fields[0].field, "field");
    assert_eq!(fields[0].binding, "field");
    assert_eq!(fields[1].field, "other");
    assert_eq!(fields[1].binding, "renamed");
    assert!(matches!(arms[2].pattern, MatchPattern::Wildcard { .. }));
}

#[test]
fn match_scrutinee_brace_is_not_misparsed_as_construction() {
    let expression = parsed("match State.First { State.First => 1, }").unwrap();
    let ExpressionKind::Match { scrutinee, .. } = expression.kind else {
        panic!("expected match");
    };
    assert!(matches!(
        scrutinee.kind,
        ExpressionKind::FieldProject { .. }
    ));

    let nested =
        parsed("match (State.Payload { value: 1, }) { State.Payload { value, } => value, }")
            .unwrap();
    let ExpressionKind::Match { scrutinee, .. } = nested.kind else {
        panic!("expected match");
    };
    assert!(matches!(
        scrutinee.kind,
        ExpressionKind::NominalConstruct { .. }
    ));
}

#[test]
fn wildcard_is_final_and_match_is_nonempty() {
    assert_eq!(
        parsed("match state { _ => 0, State.First => 1 }")
            .unwrap_err()
            .code(),
        "EXPRESSION_MATCH_WILDCARD_ORDER"
    );
    assert_eq!(
        parsed("match state {}").unwrap_err().code(),
        "EXPRESSION_MATCH_ARM"
    );
}

#[test]
fn pattern_field_and_binding_names_are_unique() {
    for source in [
        "match state { State.Item { value, value } => value }",
        "match state { State.Item { first: value, second: value } => value }",
    ] {
        assert_eq!(
            parsed(source).unwrap_err().code(),
            "EXPRESSION_DUPLICATE_PATTERN_NAME",
            "{source}"
        );
    }
}
