use super::super::super::token::{Token, TokenKind};
use super::super::trivia::{has_blank_line, has_newline};

#[derive(Clone, Copy)]
pub(super) enum Separator {
    None,
    Space,
    Line,
    BlankLine,
}

pub(super) fn separator(
    before: Option<&Token>,
    previous: Option<&Token>,
    current: &Token,
    gap: &str,
    closing_block: bool,
) -> Separator {
    let Some(previous) = previous else {
        return Separator::None;
    };
    let line = || {
        if has_blank_line(gap) {
            Separator::BlankLine
        } else {
            Separator::Line
        }
    };
    if matches!(current.kind, TokenKind::Eof) {
        return Separator::None;
    }
    if matches!(
        current.kind,
        TokenKind::RightParen | TokenKind::RightBracket
    ) && has_newline(gap)
    {
        return Separator::Line;
    }
    if closes_tightly(&current.kind, closing_block) || opens_tightly(&previous.kind) {
        return Separator::None;
    }
    if matches!(previous.kind, TokenKind::LeftBrace) || closing_block {
        return line();
    }
    if matches!(previous.kind, TokenKind::Semicolon) {
        return line();
    }
    if matches!(previous.kind, TokenKind::RightBrace) {
        return if current.word() == Some("else") {
            Separator::Space
        } else {
            line()
        };
    }
    let previous_text = spelling(previous);
    let current_text = spelling(current);
    if glue(previous_text, current_text, gap) {
        return Separator::None;
    }
    if current_text.starts_with('.') && has_newline(gap) {
        return Separator::Line;
    }
    if matches!(previous.kind, TokenKind::Comma) {
        return if has_newline(gap) {
            Separator::Line
        } else {
            Separator::Space
        };
    }
    if matches!(previous.kind, TokenKind::Colon) {
        return Separator::Space;
    }
    if is_operator(&current.kind) || is_operator(&previous.kind) {
        return if unary(previous, before, gap) {
            Separator::None
        } else {
            Separator::Space
        };
    }
    if matches!(current.kind, TokenKind::LeftBrace) {
        return Separator::Space;
    }
    if matches!(current.kind, TokenKind::LeftParen | TokenKind::LeftBracket) {
        return Separator::None;
    }
    Separator::Space
}

fn closes_tightly(kind: &TokenKind, closing_block: bool) -> bool {
    matches!(
        kind,
        TokenKind::Semicolon
            | TokenKind::Comma
            | TokenKind::RightParen
            | TokenKind::RightBracket
            | TokenKind::Colon
    ) || matches!(kind, TokenKind::RightBrace) && !closing_block
}

fn opens_tightly(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::LeftParen | TokenKind::LeftBracket | TokenKind::DollarLeftBrace
    )
}

fn is_operator(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Equals
            | TokenKind::EqualsEquals
            | TokenKind::Bang
            | TokenKind::BangEquals
            | TokenKind::LessEquals
            | TokenKind::GreaterEquals
            | TokenKind::AndAnd
            | TokenKind::OrOr
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Arrow
            | TokenKind::DotDot
    )
}

fn unary(previous: &Token, before: Option<&Token>, _gap: &str) -> bool {
    if matches!(previous.kind, TokenKind::Bang) {
        return true;
    }
    matches!(previous.kind, TokenKind::Plus | TokenKind::Minus)
        && before.is_none_or(|token| {
            is_operator(&token.kind)
                || matches!(
                    token.kind,
                    TokenKind::Less
                        | TokenKind::Greater
                        | TokenKind::LeftParen
                        | TokenKind::LeftBracket
                        | TokenKind::LeftBrace
                        | TokenKind::Comma
                        | TokenKind::Colon
                        | TokenKind::Semicolon
                )
        })
}

fn glue(previous: &str, current: &str, gap: &str) -> bool {
    previous.starts_with('.')
        || previous.ends_with('.')
        || current.starts_with('.') && !has_newline(gap)
        || previous == "#" && current == "{"
        || previous == "=" && current == ">"
        || gap.is_empty() && matches!(previous, "<" | ">")
        || gap.is_empty() && matches!(current, "<" | ">")
}

fn spelling(token: &Token) -> &str {
    match &token.kind {
        TokenKind::Word(value) | TokenKind::Number(value) | TokenKind::Color(value) => value,
        TokenKind::String(_) => "\"\"",
        TokenKind::LocalId(_) => "@id",
        TokenKind::LeftBrace => "{",
        TokenKind::RightBrace => "}",
        TokenKind::LeftParen => "(",
        TokenKind::RightParen => ")",
        TokenKind::LeftBracket => "[",
        TokenKind::RightBracket => "]",
        TokenKind::Colon => ":",
        TokenKind::Arrow => "->",
        TokenKind::DotDot => "..",
        TokenKind::Semicolon => ";",
        TokenKind::Comma => ",",
        TokenKind::Equals => "=",
        TokenKind::EqualsEquals => "==",
        TokenKind::Bang => "!",
        TokenKind::BangEquals => "!=",
        TokenKind::Less => "<",
        TokenKind::LessEquals => "<=",
        TokenKind::Greater => ">",
        TokenKind::GreaterEquals => ">=",
        TokenKind::AndAnd => "&&",
        TokenKind::OrOr => "||",
        TokenKind::Plus => "+",
        TokenKind::Minus => "-",
        TokenKind::Star => "*",
        TokenKind::Slash => "/",
        TokenKind::DollarLeftBrace => "${",
        TokenKind::Eof => "",
    }
}
