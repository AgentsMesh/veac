use super::Parser;
use crate::authoring::{ItemDecl, ModifierDecl, RecordSpan, Span};

impl Parser {
    pub(super) fn item(&mut self) -> Option<ItemDecl> {
        let start = self.required_word("item")?;
        let id = self.identifier("item")?;
        self.left_brace()?;
        let mut source = None;
        let mut record = None;
        let mut state = None;
        let mut mapping = None;
        let mut template_slot = None;
        let mut modifiers = Vec::new();
        while !self.at_right_brace() && !self.at_eof() {
            if self.at_word("source") {
                let span = self.current().span;
                let value = self.source();
                assign(self, "source", &mut source, value, span);
            } else if self.at_word("record") {
                let span = self.current().span;
                let value = self.record_span();
                assign(self, "record", &mut record, value, span);
            } else if self.at_word("state") {
                let span = self.required_word("state")?;
                let value = self
                    .semantic_block()
                    .and_then(|block| super::timeline_state::item(self, block));
                assign(self, "item state", &mut state, value, span);
            } else if self.at_word("mapping") {
                let span = self.current().span;
                let value = self.mapping();
                assign(self, "mapping", &mut mapping, value, span);
            } else if self.at_word("template-slot") {
                let span = self.current().span;
                let value = self.template_slot();
                assign(self, "template-slot", &mut template_slot, value, span);
            } else if self.at_word("modifiers") {
                modifiers.extend(self.modifiers()?);
            } else {
                let span = self.current().span;
                self.error(
                    "AUTHORING_ITEM_MEMBER",
                    "item member must be source, record, state, mapping, template-slot, or modifiers"
                        .to_owned(),
                    span,
                );
                self.advance();
                self.recover_declaration();
            }
        }
        let end = self.right_brace()?;
        let span = start.join(end);
        let source = required(self, "source", source, span)?;
        Some(ItemDecl {
            id,
            source,
            record: required(self, "record", record, span)?,
            state,
            mapping,
            template_slot,
            modifiers,
            span,
        })
    }

    pub(super) fn record_span(&mut self) -> Option<RecordSpan> {
        let start = self.required_word("record")?;
        self.left_brace()?;
        let mut at = None;
        let mut duration = None;
        while !self.at_right_brace() && !self.at_eof() {
            let field = self.identifier("span field")?;
            let value = self.number("span time");
            self.semicolon();
            match field.value.as_str() {
                "at" => assign(self, "span at", &mut at, value, field.span),
                "duration" => assign(self, "span duration", &mut duration, value, field.span),
                _ => self.error(
                    "AUTHORING_SPAN_FIELD",
                    "span field must be at or duration".to_owned(),
                    field.span,
                ),
            }
        }
        let end = self.right_brace()?;
        let span = start.join(end);
        Some(RecordSpan {
            at: required(self, "span at", at, span)?,
            duration: required(self, "span duration", duration, span)?,
            span,
        })
    }

    pub(super) fn modifiers(&mut self) -> Option<Vec<ModifierDecl>> {
        self.required_word("modifiers")?;
        self.left_brace()?;
        let mut values = Vec::new();
        while !self.at_right_brace() && !self.at_eof() {
            let modifier_kind = self.identifier("modifier kind")?;
            let id = self.identifier("modifier")?;
            let body = self.semantic_block()?;
            if let Some(value) = self.modifier(modifier_kind, id, body) {
                values.push(value);
            }
        }
        self.right_brace()?;
        Some(values)
    }
}

pub(super) fn assign<T>(
    parser: &mut Parser,
    name: &'static str,
    slot: &mut Option<T>,
    value: Option<T>,
    span: Span,
) {
    if slot.is_some() {
        parser.duplicate(name, span);
    } else if let Some(value) = value {
        *slot = Some(value);
    }
}

pub(super) fn required<T>(
    parser: &mut Parser,
    name: &'static str,
    value: Option<T>,
    span: Span,
) -> Option<T> {
    if value.is_none() {
        parser.error(
            "AUTHORING_REQUIRED_FIELD",
            format!("block requires {name}"),
            span,
        );
    }
    value
}
