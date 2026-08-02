use crate::authoring::lexer::{lex, TokenKind};

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
fn lexer_decodes_every_supported_string_escape_and_preserves_unknown_escapes() {
    let (tokens, diagnostics) = lex(r#""line\nreturn\rtab\tquote\"slash\\unknown\q""#);
    assert!(diagnostics.is_empty());
    assert_eq!(
        tokens[0].kind,
        TokenKind::String("line\nreturn\rtab\tquote\"slash\\unknownq".into())
    );
    assert_eq!(tokens[1].kind, TokenKind::Eof);
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
