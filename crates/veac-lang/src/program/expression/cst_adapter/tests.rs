use super::{lex_parts, ExpressionSyntax};
use crate::authoring::Span;
use crate::program::expression::lexer;
use crate::program::syntax_document::SyntaxSlice;

fn adapted(source: &str) -> Result<Vec<lexer::Token>, super::ExpressionError> {
    let document = crate::program::lexer::lex_document("adapter.veac", source).unwrap();
    let eof = document.tokens().len() - 1;
    lex_parts(
        document.source(),
        &document.tokens()[0..eof],
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
fn cst_adapter_matches_expression_tokens_across_the_surface() {
    for source in [
        "module.path(name: 1.5MS, other: .5)",
        "true false let var set if else by fn for in match animate",
        "+ - * / ! != < <= > >= = == && || -> =>",
        "#{\"escaped\\n\": #AABBCCDD, key: [1s, 2px, 3DEG]}",
        "#abcdef{} #abcdefaa{}",
        "match value { Type.Item => value.field }",
        "a--b a->b name. .field 1.foo 1.0.5",
        "value().foo_bar.baz_qux value().foo-bar.baz a_b.c 1ms.foo_bar",
        "=>= =>== a->=b a->==b a->=>b",
        "1/* keep */+// line\n2",
        "fn(value: int) -> int effect pure { value }",
        "{ project(identifier(\"database\"), project_settings(600))\n    .with_sequence(timeline).entry(timeline) }",
        "{ let timeline = sequence(identifier(\"main\"), \"数据库\", sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)); project(identifier(\"database\"), project_settings(600)).with_sequence(timeline).entry(timeline) }",
    ] {
        assert_eq!(
            adapted(source).unwrap(),
            lexer::lex(source).unwrap(),
            "{source}"
        );
    }
}

#[test]
fn cst_adapter_matches_expression_lexical_failures() {
    for source in [
        "#abc",
        "#abcdefg",
        "#gggggg",
        "#abc{}",
        "# /* gap */ {",
        "1_value",
        "\"bad\\q\"",
        "\"bad\ncontrol\"",
        "\"\\u{}\"",
        "标题",
        "1é",
        "@local",
        "${value}",
    ] {
        let expected = lexer::lex(source).unwrap_err();
        let actual = adapted(source).unwrap_err();
        assert_eq!(actual.code(), expected.code(), "{source}");
        assert_eq!(actual.message(), expected.message(), "{source}");
        assert_eq!(actual.span(), expected.span(), "{source}");
    }
}

#[test]
fn cst_absolute_spans_are_normalized_to_the_slice_origin() {
    let expression = "module.path(1s)";
    let source = format!("标题 ignored\n{expression}");
    let document = crate::program::lexer::lex_document("unicode.veac", &source).unwrap();
    let start = source.find(expression).unwrap();
    let token_start = document
        .tokens()
        .iter()
        .position(|token| token.span.start == start)
        .unwrap();
    let eof = document.tokens().len() - 1;
    let actual = lex_parts(
        document.source(),
        &document.tokens()[token_start..eof],
        &SyntaxSlice {
            span: Span {
                start,
                end: source.len(),
            },
            tokens: token_start..eof,
        },
    )
    .unwrap();
    assert_eq!(actual, lexer::lex(expression).unwrap());
}

#[test]
fn expression_syntax_owns_its_document_backing() {
    let source = "{ 1.0 + 2.0 }";
    let syntax = {
        let document = crate::program::lexer::lex_document("owned.veac", source).unwrap();
        let eof = document.tokens().len() - 1;
        ExpressionSyntax::new(
            &document,
            &SyntaxSlice {
                span: Span {
                    start: 0,
                    end: source.len(),
                },
                tokens: 0..eof,
            },
        )
    };

    let expression = syntax.parse().unwrap();
    assert_eq!(expression.span, 0..source.len());
}
