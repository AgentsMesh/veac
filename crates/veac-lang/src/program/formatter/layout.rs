mod rules;
mod writer;

use super::super::token::{Token, TokenKind};
use rules::separator;
use writer::Writer;

pub(super) fn format(source: &str, tokens: &[Token]) -> String {
    let mut writer = Writer::default();
    let mut previous: Option<&Token> = None;
    let mut before_previous: Option<&Token> = None;
    let mut offset = 0;
    for token in tokens {
        let eof = matches!(token.kind, TokenKind::Eof);
        let closing_block =
            matches!(token.kind, TokenKind::RightBrace) && writer.closes_regular_block();
        if closing_block {
            writer.close_regular_block();
        }
        let closing_continuation =
            matches!(token.kind, TokenKind::RightParen | TokenKind::RightBracket);
        if closing_continuation {
            writer.close_continuation();
        }
        let gap = &source[offset..token.span.start];
        writer.trivia(
            gap,
            separator(before_previous, previous, token, gap, closing_block),
            previous.is_some(),
            !eof,
        );
        if eof {
            break;
        }
        writer.token(&source[token.span.start..token.span.end]);
        match token.kind {
            TokenKind::LeftBrace => writer.open_regular_block(),
            TokenKind::DollarLeftBrace => writer.open_interpolation(),
            TokenKind::RightBrace => writer.close_brace(),
            TokenKind::LeftParen | TokenKind::LeftBracket => writer.open_continuation(),
            TokenKind::RightParen | TokenKind::RightBracket => {}
            _ => {}
        }
        offset = token.span.end;
        before_previous = previous;
        previous = Some(token);
    }
    writer.finish()
}
