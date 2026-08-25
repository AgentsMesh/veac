use crate::program::token::TokenKind as OuterKind;

use super::super::lexer::TokenKind;

pub(super) fn simple(kind: &OuterKind) -> Option<TokenKind> {
    Some(match kind {
        OuterKind::LeftBrace => TokenKind::LeftBrace,
        OuterKind::RightBrace => TokenKind::RightBrace,
        OuterKind::LeftParen => TokenKind::LeftParen,
        OuterKind::RightParen => TokenKind::RightParen,
        OuterKind::LeftBracket => TokenKind::LeftBracket,
        OuterKind::RightBracket => TokenKind::RightBracket,
        OuterKind::Colon => TokenKind::Colon,
        OuterKind::Arrow => TokenKind::Arrow,
        OuterKind::DotDot => TokenKind::DotDot,
        OuterKind::Semicolon => TokenKind::Semicolon,
        OuterKind::Comma => TokenKind::Comma,
        OuterKind::Equals => TokenKind::Equal,
        OuterKind::EqualsEquals => TokenKind::EqualEqual,
        OuterKind::Bang => TokenKind::Bang,
        OuterKind::BangEquals => TokenKind::BangEqual,
        OuterKind::Less => TokenKind::Less,
        OuterKind::LessEquals => TokenKind::LessEqual,
        OuterKind::Greater => TokenKind::Greater,
        OuterKind::GreaterEquals => TokenKind::GreaterEqual,
        OuterKind::AndAnd => TokenKind::AndAnd,
        OuterKind::OrOr => TokenKind::OrOr,
        OuterKind::Plus => TokenKind::Plus,
        OuterKind::Minus => TokenKind::Minus,
        OuterKind::Star => TokenKind::Star,
        OuterKind::Slash => TokenKind::Slash,
        _ => return None,
    })
}
