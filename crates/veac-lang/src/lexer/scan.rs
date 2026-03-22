use crate::error::{ErrorKind, VeacError};
use crate::token::{Token, TokenKind};

use super::{is_smpte, Lexer};

/// Scanning methods for individual token types.
impl Lexer<'_> {
    pub(crate) fn lex_line_comment(&mut self) -> Result<Token, VeacError> {
        self.advance(); // skip first /
        self.advance(); // skip second /

        while let Some(ch) = self.current() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }

        self.next_token()
    }

    pub(crate) fn lex_string(
        &mut self,
        start: usize,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token, VeacError> {
        self.advance(); // skip opening "
        let content_start = self.pos;

        loop {
            match self.current() {
                None => {
                    return Err(VeacError::new(
                        ErrorKind::UnterminatedString,
                        "unterminated string literal",
                        Some(self.span_from(start, start_line, start_col)),
                    )
                    .with_hint("add a closing `\"` to terminate the string"));
                }
                Some('"') => {
                    let content: String = self.chars[content_start..self.pos].iter().collect();
                    self.advance();
                    let kind = if is_smpte(&content) {
                        TokenKind::Smpte(content)
                    } else {
                        TokenKind::StringLit(content)
                    };
                    return Ok(Token::new(
                        kind,
                        self.span_from(start, start_line, start_col),
                    ));
                }
                Some('\\') => {
                    self.advance();
                    self.advance();
                }
                Some(_) => {
                    self.advance();
                }
            }
        }
    }

    pub(crate) fn lex_color(
        &mut self,
        start: usize,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token, VeacError> {
        self.advance(); // skip #

        let hex_start = self.pos;
        while let Some(ch) = self.current() {
            if ch.is_ascii_hexdigit() {
                self.advance();
            } else {
                break;
            }
        }

        let hex: String = self.chars[hex_start..self.pos].iter().collect();
        if hex.len() != 6 && hex.len() != 8 {
            return Err(VeacError::new(
                ErrorKind::InvalidColor,
                format!("invalid color literal `#{hex}`, expected 6 or 8 hex digits"),
                Some(self.span_from(start, start_line, start_col)),
            )
            .with_hint("use format #RRGGBB or #RRGGBBAA"));
        }

        Ok(Token::new(
            TokenKind::ColorLit(hex),
            self.span_from(start, start_line, start_col),
        ))
    }

    pub(crate) fn lex_number(
        &mut self,
        start: usize,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token, VeacError> {
        let num_start = self.pos;
        let mut has_dot = false;

        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.' && !has_dot && self.peek().is_some_and(|n| n.is_ascii_digit()) {
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }

        let num_str: String = self.chars[num_start..self.pos].iter().collect();
        let span = self.span_from(start, start_line, start_col);

        self.try_lex_time_suffix(&num_str, has_dot, span)
    }

    fn try_lex_time_suffix(
        &mut self,
        num_str: &str,
        has_dot: bool,
        span: crate::token::Span,
    ) -> Result<Token, VeacError> {
        if let Some(ch) = self.current() {
            if ch == 's' {
                self.advance();
                let val: f64 = num_str.parse().map_err(|_| {
                    VeacError::new(
                        ErrorKind::InvalidTimeLiteral,
                        format!("invalid time value `{num_str}s`"),
                        Some(span),
                    )
                })?;
                return Ok(Token::new(TokenKind::TimeSec(val), span));
            }
            if ch == 'm' && self.peek() == Some('s') {
                self.advance();
                self.advance();
                let val: f64 = num_str.parse().map_err(|_| {
                    VeacError::new(
                        ErrorKind::InvalidTimeLiteral,
                        format!("invalid time value `{num_str}ms`"),
                        Some(span),
                    )
                })?;
                return Ok(Token::new(TokenKind::TimeMs(val), span));
            }
            if ch == 'f' && !has_dot {
                self.advance();
                let val: u64 = num_str.parse().map_err(|_| {
                    VeacError::new(
                        ErrorKind::InvalidTimeLiteral,
                        format!("invalid frame value `{num_str}f`"),
                        Some(span),
                    )
                })?;
                return Ok(Token::new(TokenKind::TimeFrames(val), span));
            }
        }

        // Plain number
        if has_dot {
            let val: f64 = num_str.parse().map_err(|_| {
                VeacError::new(
                    ErrorKind::InvalidNumber,
                    format!("invalid float literal `{num_str}`"),
                    Some(span),
                )
            })?;
            Ok(Token::new(TokenKind::FloatLit(val), span))
        } else {
            let val: i64 = num_str.parse().map_err(|_| {
                VeacError::new(
                    ErrorKind::InvalidNumber,
                    format!("invalid integer literal `{num_str}`"),
                    Some(span),
                )
            })?;
            Ok(Token::new(TokenKind::IntLit(val), span))
        }
    }

    /// Lex a negative number: `-` followed by digits.
    pub(crate) fn lex_negative_number(
        &mut self,
        start: usize,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token, VeacError> {
        self.advance(); // skip '-'
        let tok = self.lex_number(start, start_line, start_col)?;
        // Negate the parsed value.
        let negated = match tok.kind {
            TokenKind::IntLit(n) => TokenKind::IntLit(-n),
            TokenKind::FloatLit(n) => TokenKind::FloatLit(-n),
            TokenKind::TimeSec(n) => TokenKind::TimeSec(-n),
            TokenKind::TimeMs(n) => TokenKind::TimeMs(-n),
            other => other,
        };
        Ok(Token::new(negated, tok.span))
    }

    pub(crate) fn lex_ident_or_keyword(
        &mut self,
        start: usize,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token, VeacError> {
        let ident_start = self.pos;

        while let Some(ch) = self.current() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let ident: String = self.chars[ident_start..self.pos].iter().collect();
        let kind = match ident.as_str() {
            "project" => TokenKind::Project,
            "asset" => TokenKind::Asset,
            "timeline" => TokenKind::Timeline,
            "track" => TokenKind::Track,
            "clip" => TokenKind::Clip,
            "text" => TokenKind::Text,
            "transition" => TokenKind::Transition,
            "let" => TokenKind::Let,
            "include" => TokenKind::Include,
            "video" => TokenKind::Video,
            "audio" => TokenKind::Audio,
            "image" => TokenKind::Image,
            "true" => TokenKind::BoolLit(true),
            "false" => TokenKind::BoolLit(false),
            _ => TokenKind::Ident(ident),
        };

        Ok(Token::new(
            kind,
            self.span_from(start, start_line, start_col),
        ))
    }
}
