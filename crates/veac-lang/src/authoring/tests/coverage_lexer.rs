use crate::authoring::lexer::{lex_with_limits, Token, TokenKind};

fn lex(source: &str) -> (Vec<Token>, Vec<crate::authoring::Diagnostic>) {
    lex_with_limits(source, usize::MAX, usize::MAX)
}

#[test]
fn lexer_classifies_delimiters_signed_numbers_words_colors_and_eof() {
    let (tokens, diagnostics) =
        lex(" // ignored\n{ }; alpha -12.5ms +7fps .member #AABBCC #11223344");
    assert!(diagnostics.is_empty());
    assert_eq!(
        tokens
            .into_iter()
            .map(|value| value.kind)
            .collect::<Vec<_>>(),
        [
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::Semicolon,
            TokenKind::Word("alpha".into()),
            TokenKind::Number("-12.5ms".into()),
            TokenKind::Number("+7fps".into()),
            TokenKind::Word(".member".into()),
            TokenKind::Color("#aabbcc".into()),
            TokenKind::Color("#11223344".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexer_decodes_supported_string_escapes_and_rejects_unknown_escapes() {
    let (tokens, diagnostics) = lex(r#""line\nreturn\rtab\tquote\"slash\\""#);
    assert!(diagnostics.is_empty());
    assert_eq!(
        tokens[0].kind,
        TokenKind::String("line\nreturn\rtab\tquote\"slash\\".into())
    );
    assert_eq!(tokens[1].kind, TokenKind::Eof);

    let (_, diagnostics) = lex(r#""unknown\q""#);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "AUTHORING_LEX_STRING_ESCAPE");

    let (_, diagnostics) = lex("\"raw\ncontrol\"");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "AUTHORING_LEX_STRING_CONTROL");
}

#[test]
fn lexer_reports_unterminated_strings_and_each_unexpected_character_span() {
    let (tokens, diagnostics) = lex("@, ` \"unterminated");
    assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
    assert_eq!(diagnostics.len(), 4);
    assert_eq!(
        diagnostics
            .iter()
            .map(|value| value.code)
            .collect::<Vec<_>>(),
        [
            "AUTHORING_LEX_CHARACTER",
            "AUTHORING_LEX_CHARACTER",
            "AUTHORING_LEX_CHARACTER",
            "AUTHORING_LEX_STRING",
        ]
    );
    assert_eq!(
        diagnostics[0].span,
        crate::authoring::Span { start: 0, end: 1 }
    );
    assert!(diagnostics[3].message.contains("unterminated string"));
}

#[test]
fn lexer_accepts_block_comments_and_reports_an_unclosed_comment() {
    let (tokens, diagnostics) = lex("/* before */ word/* middle */other// after");
    assert!(diagnostics.is_empty());
    assert_eq!(tokens[0].kind, TokenKind::Word("word".into()));
    assert_eq!(tokens[1].kind, TokenKind::Word("other".into()));

    let (_, diagnostics) = lex("/* never closed");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "AUTHORING_LEX_BLOCK_COMMENT");
}

#[test]
fn lexer_enforces_source_and_token_allocation_budgets() {
    let (_, diagnostics) = lex_with_limits("too large", 4, 100);
    assert_eq!(diagnostics[0].code, "AUTHORING_SOURCE_LIMIT");

    let (tokens, diagnostics) = lex_with_limits("one two three", 100, 2);
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "AUTHORING_TOKEN_LIMIT");

    for errors in [
        crate::authoring::parser::parse_with_limits("too large", 4, 100).unwrap_err(),
        crate::authoring::parser::parse_with_limits("one two three", 100, 2).unwrap_err(),
    ] {
        assert!(errors
            .as_slice()
            .iter()
            .any(|error| error.code.ends_with("_LIMIT")));
    }
}

#[test]
fn lexer_bounds_diagnostics_from_invalid_characters() {
    let (_, diagnostics) = lex(&"@".repeat(1_000));
    assert_eq!(diagnostics.len(), 256);
    assert_eq!(
        diagnostics.last().unwrap().code,
        "AUTHORING_DIAGNOSTIC_LIMIT"
    );
}
