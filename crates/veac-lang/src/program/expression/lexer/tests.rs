use super::{lex, TokenKind};

fn significant(source: &str) -> Vec<TokenKind> {
    lex(source)
        .unwrap()
        .into_iter()
        .map(|token| token.kind)
        .filter(|kind| kind != &TokenKind::Eof)
        .collect()
}

#[test]
fn range_separator_uses_longest_match_at_numeric_boundaries() {
    for source in ["1..10", "1.0..2", ".5..2", "1 .. 10"] {
        let kinds = significant(source);
        assert_eq!(kinds.len(), 3, "{source}");
        assert!(matches!(kinds[0], TokenKind::Number { .. }), "{source}");
        assert_eq!(kinds[1], TokenKind::DotDot, "{source}");
        assert!(matches!(kinds[2], TokenKind::Number { .. }), "{source}");
    }
}

#[test]
fn range_words_and_symbol_boundaries_remain_unambiguous() {
    assert_eq!(significant("start..end by step")[1], TokenKind::DotDot);
    assert!(significant("start..end by step").contains(&TokenKind::By));
    assert_eq!(significant("."), [TokenKind::Dot]);
    assert_eq!(
        significant("name."),
        [TokenKind::Symbol("name".to_owned()), TokenKind::Dot]
    );
}

#[test]
fn closure_introducer_and_arrow_are_closed_tokens() {
    let kinds = significant("fn(value: int) -> int effect pure { value }");
    assert_eq!(kinds[0], TokenKind::Fn);
    assert!(kinds.contains(&TokenKind::Arrow));
    assert!(!kinds.contains(&TokenKind::Minus));
}

#[test]
fn field_match_and_arm_tokens_use_longest_match() {
    let kinds = significant("match value { Type.Item => value.field }");
    assert_eq!(kinds[0], TokenKind::Match);
    assert_eq!(
        kinds.iter().filter(|kind| **kind == TokenKind::Dot).count(),
        2
    );
    assert!(kinds.contains(&TokenKind::FatArrow));
    assert!(!kinds.contains(&TokenKind::Equal));
}
