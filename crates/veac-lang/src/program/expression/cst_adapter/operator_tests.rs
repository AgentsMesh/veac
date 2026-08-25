use super::lex_parts;
use crate::authoring::Span;
use crate::program::expression::lexer;
use crate::program::syntax_document::SyntaxSlice;
use crate::program::token::{Token, TokenKind};

fn adapted(source: &str) -> Result<Vec<lexer::Token>, super::ExpressionError> {
    let document = crate::program::lexer::lex_document("operators.veac", source).unwrap();
    let eof = document.tokens().len() - 1;
    lex_parts(
        document.source(),
        &document.tokens()[..eof],
        &SyntaxSlice {
            span: Span {
                start: 0,
                end: source.len(),
            },
            tokens: 0..eof,
        },
    )
}

#[test]
fn contiguous_operator_runs_match_the_expression_lexer() {
    for source in [
        "=+-*/!!===<<=>>=&&||",
        "=->=>",
        "=// line comment\n1",
        "=/* block comment */1",
    ] {
        assert_eq!(
            adapted(source).unwrap(),
            lexer::lex(source).unwrap(),
            "{source}"
        );
    }
}

#[test]
fn incomplete_boolean_operators_preserve_the_lexical_error_contract() {
    for source in ["=&", "=|"] {
        let actual = lex_parts(
            source,
            &[Token {
                kind: TokenKind::Equals,
                span: Span { start: 0, end: 1 },
            }],
            &SyntaxSlice {
                span: Span {
                    start: 0,
                    end: source.len(),
                },
                tokens: 0..1,
            },
        )
        .unwrap_err();
        let value = source.chars().nth(1).unwrap();
        assert_eq!(actual.code(), "EXPRESSION_LEX_OPERATOR", "{source}");
        assert_eq!(
            actual.message(),
            format!("operator `{value}` must be followed by `{value}`"),
            "{source}"
        );
        assert_eq!(actual.span(), 1..2, "{source}");
    }
}
