use super::Parser;
use crate::authoring::lexer::TokenKind;
use crate::authoring::{SemanticBlock, SemanticEntry, SemanticValue, Spanned, TypedReference};

impl Parser {
    pub(super) fn recover_declaration(&mut self) {
        if self.at_left_brace() {
            let mut depth = 0usize;
            while !self.at_eof() {
                let token = self.advance();
                match token.kind {
                    TokenKind::LeftBrace => depth += 1,
                    TokenKind::RightBrace if depth == 1 => break,
                    TokenKind::RightBrace => depth = depth.saturating_sub(1),
                    _ => {}
                }
            }
        } else {
            while !self.at_eof() && !self.at_semicolon() && !self.at_right_brace() {
                self.advance();
            }
            if self.at_semicolon() {
                self.advance();
            }
        }
    }

    pub(super) fn semantic_block(&mut self) -> Option<SemanticBlock> {
        let start = self.left_brace()?;
        let mut entries = Vec::new();
        while !self.at_right_brace() && !self.at_eof() {
            if let Some(entry) = self.semantic_entry() {
                entries.push(entry);
            } else {
                self.recover_declaration();
            }
        }
        let end = self.right_brace()?;
        Some(SemanticBlock {
            entries,
            span: start.join(end),
        })
    }

    pub(super) fn typed_reference(&mut self) -> Option<TypedReference> {
        let kind = self.identifier("reference type")?;
        let id = self.identifier("reference target")?;
        Some(TypedReference {
            span: kind.span.join(id.span),
            kind,
            id,
        })
    }

    fn semantic_entry(&mut self) -> Option<SemanticEntry> {
        let name = self.identifier("semantic field")?;
        let mut values = Vec::new();
        while !self.at_semicolon()
            && !self.at_left_brace()
            && !self.at_right_brace()
            && !self.at_eof()
        {
            values.push(self.semantic_value()?);
        }
        let (block, end) = if self.at_left_brace() {
            let block = self.semantic_block()?;
            let end = block.span;
            (Some(block), end)
        } else {
            (None, self.semicolon()?)
        };
        Some(SemanticEntry {
            span: name.span.join(end),
            name,
            values,
            block,
        })
    }

    fn semantic_value(&mut self) -> Option<SemanticValue> {
        let token = self.advance();
        let value = match token.kind {
            TokenKind::String(value) => SemanticValue::String(Spanned {
                value,
                span: token.span,
            }),
            TokenKind::Color(value) => SemanticValue::Color(Spanned {
                value,
                span: token.span,
            }),
            TokenKind::Number(raw) => SemanticValue::Number(crate::authoring::NumberLiteral {
                raw,
                span: token.span,
            }),
            TokenKind::Word(value) if value == "true" || value == "false" => {
                SemanticValue::Boolean(Spanned {
                    value: value == "true",
                    span: token.span,
                })
            }
            TokenKind::Word(value) => SemanticValue::Identifier(Spanned {
                value,
                span: token.span,
            }),
            _ => {
                self.error(
                    "AUTHORING_SEMANTIC_VALUE",
                    "expected string, number, boolean, identifier, or typed reference".to_owned(),
                    token.span,
                );
                return None;
            }
        };
        Some(value)
    }
}
