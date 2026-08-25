use super::super::lexer::TokenKind;
use super::super::ExpressionError;
use super::Adapter;

impl Adapter<'_> {
    pub(super) fn operators(&mut self, mut at: usize) -> Result<usize, ExpressionError> {
        while at < self.end {
            let value = self.character(at);
            let next = self.next_character(at);
            let (kind, bytes) = match value {
                '+' => (TokenKind::Plus, 1),
                '-' if next == Some('>') => (TokenKind::Arrow, 2),
                '-' => (TokenKind::Minus, 1),
                '*' => (TokenKind::Star, 1),
                '/' if matches!(next, Some('/' | '*')) => break,
                '/' => (TokenKind::Slash, 1),
                '!' if next == Some('=') => (TokenKind::BangEqual, 2),
                '!' => (TokenKind::Bang, 1),
                '=' if next == Some('>') => (TokenKind::FatArrow, 2),
                '=' if next == Some('=') => (TokenKind::EqualEqual, 2),
                '=' => (TokenKind::Equal, 1),
                '<' if next == Some('=') => (TokenKind::LessEqual, 2),
                '<' => (TokenKind::Less, 1),
                '>' if next == Some('=') => (TokenKind::GreaterEqual, 2),
                '>' => (TokenKind::Greater, 1),
                '&' if next == Some('&') => (TokenKind::AndAnd, 2),
                '|' if next == Some('|') => (TokenKind::OrOr, 2),
                '&' | '|' => return Err(self.operator_error(value, at)),
                _ => break,
            };
            self.push(kind, at, at + bytes);
            at += bytes;
        }
        Ok(at)
    }

    fn operator_error(&self, value: char, start: usize) -> ExpressionError {
        ExpressionError::new(
            "EXPRESSION_LEX_OPERATOR",
            format!("operator `{value}` must be followed by `{value}`"),
            self.local(start, start + value.len_utf8()),
        )
    }
}
