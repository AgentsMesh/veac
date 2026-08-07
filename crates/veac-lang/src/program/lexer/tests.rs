use super::lex_with_limit;
use crate::program::token::TokenKind;

#[test]
fn lexer_emits_function_signature_punctuation() {
    let source = "fn keep(values: list<int>) -> list<int> { [values] }";
    let tokens = super::lex("function.veac", source).unwrap();
    assert!(tokens.iter().any(|token| token.kind == TokenKind::Colon));
    assert!(tokens.iter().any(|token| token.kind == TokenKind::Arrow));
    assert!(tokens
        .iter()
        .any(|token| token.kind == TokenKind::LeftBracket));
    assert!(tokens
        .iter()
        .any(|token| token.kind == TokenKind::RightBracket));
    assert!(!tokens.iter().any(|token| token.kind == TokenKind::Minus));
}

#[test]
fn lexer_uses_longest_match_for_expression_operators() {
    let source = "! != < <= > >= = == && ||";
    let tokens = super::lex("operators.veac", source).unwrap();
    let kinds = tokens
        .into_iter()
        .map(|token| token.kind)
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Bang,
            TokenKind::BangEquals,
            TokenKind::Less,
            TokenKind::LessEquals,
            TokenKind::Greater,
            TokenKind::GreaterEquals,
            TokenKind::Equals,
            TokenKind::EqualsEquals,
            TokenKind::AndAnd,
            TokenKind::OrOr,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn operator_spans_remain_utf8_byte_offsets_and_comments_are_ignored() {
    let source = "标题 != true // && }\n== /* || */ false";
    let tokens = super::lex("unicode.veac", source).unwrap();
    let not_equal = tokens
        .iter()
        .find(|token| token.kind == TokenKind::BangEquals)
        .unwrap();
    assert_eq!((not_equal.span.start, not_equal.span.end), (7, 9));
    assert!(tokens
        .iter()
        .any(|token| token.kind == TokenKind::EqualsEquals));
    assert!(!tokens.iter().any(|token| token.kind == TokenKind::AndAnd));
    assert!(!tokens.iter().any(|token| token.kind == TokenKind::OrOr));
}

#[test]
fn isolated_boolean_operator_prefixes_are_diagnostics() {
    let errors = super::lex("invalid.veac", "& |").unwrap_err();
    assert_eq!(errors.len(), 2);
    assert!(errors.iter().all(|error| error.code == "PROGRAM_LEX_TOKEN"));
}

#[test]
fn lexer_stops_allocating_tokens_at_its_configured_budget() {
    let errors = lex_with_limit("large.veac", "one two three", 2).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, "PROGRAM_TOKEN_LIMIT");
}

#[test]
fn lexer_bounds_diagnostics_from_invalid_characters() {
    let errors = super::lex("invalid.veac", &"?".repeat(1_000)).unwrap_err();
    assert_eq!(errors.len(), 256);
    assert_eq!(errors.last().unwrap().code, "PROGRAM_DIAGNOSTIC_LIMIT");
}

#[test]
fn paired_operators_each_consume_one_token_from_the_budget() {
    let errors = lex_with_limit("large.veac", "&& ||", 1).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, "PROGRAM_TOKEN_LIMIT");
}

#[test]
fn range_separator_survives_compact_program_expression_tokens() {
    for source in ["1..10", "1.0..2", ".5..2", "start..end"] {
        let tokens = super::lex("range.veac", source).unwrap();
        assert_eq!(
            tokens
                .iter()
                .filter(|token| token.kind == TokenKind::DotDot)
                .count(),
            1,
            "{source}"
        );
        assert_eq!(
            &source[tokens[1].span.start..tokens[1].span.end],
            "..",
            "{source}"
        );
    }
}

#[test]
fn function_body_range_is_tokenized_without_source_corruption() {
    let source = "fn indices() -> range<int> { { 0..10 by 2 } }";
    let tokens = super::lex("function.veac", source).unwrap();
    assert!(tokens.iter().any(|token| token.kind == TokenKind::DotDot));
    assert!(tokens
        .iter()
        .any(|token| token.word().is_some_and(|word| word == "by")));
}
