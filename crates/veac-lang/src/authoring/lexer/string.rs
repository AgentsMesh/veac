use super::{Diagnostic, Lexer, Span, TokenKind};

impl Lexer<'_> {
    pub(super) fn string(&mut self, start: usize) {
        self.advance();
        let mut value = String::new();
        while let Some(character) = self.current() {
            match character {
                '"' => {
                    self.advance();
                    self.push(TokenKind::String(value), start, self.offset);
                    return;
                }
                '\\' => {
                    let escape_start = self.offset;
                    self.advance();
                    match crate::string_codec::decode_escape(&self.source[self.offset..]) {
                        Ok(decoded) => {
                            value.push(decoded.character);
                            let end = self.offset + decoded.bytes;
                            while self.offset < end {
                                self.advance();
                            }
                        }
                        Err(message) => {
                            self.string_error("AUTHORING_LEX_STRING_ESCAPE", message, escape_start);
                            return;
                        }
                    }
                }
                value_char if value_char.is_control() => {
                    self.string_error(
                        "AUTHORING_LEX_STRING_CONTROL",
                        "control characters must use an escape sequence".to_owned(),
                        self.offset,
                    );
                    return;
                }
                value_char => {
                    value.push(value_char);
                    self.advance();
                }
            }
        }
        self.report(Diagnostic {
            code: "AUTHORING_LEX_STRING",
            message: "unterminated string literal".to_owned(),
            span: Span {
                start,
                end: self.offset,
            },
        });
    }

    fn string_error(&mut self, code: &'static str, message: String, start: usize) {
        let end = self.offset + self.current().map_or(0, char::len_utf8);
        self.report(Diagnostic {
            code,
            message,
            span: Span { start, end },
        });
        self.skip_string();
    }

    fn skip_string(&mut self) {
        while let Some(character) = self.current() {
            self.advance();
            if character == '"' {
                break;
            }
        }
    }
}
