use super::item::assign;
use super::Parser;
use crate::authoring::{MappingDecl, MappingKey, Span};

impl Parser {
    pub(super) fn mapping(&mut self) -> Option<MappingDecl> {
        let start = self.required_word("mapping")?;
        let kind = self.identifier("mapping type")?;
        match kind.value.as_str() {
            "linear" => self.linear_mapping(start),
            "curve" => self.curve_mapping(start),
            "freeze" => self.freeze_mapping(start),
            _ => {
                self.error(
                    "AUTHORING_MAPPING_TYPE",
                    "mapping type must be linear, curve, or freeze".to_owned(),
                    kind.span,
                );
                self.recover_declaration();
                None
            }
        }
    }

    fn linear_mapping(&mut self, start: Span) -> Option<MappingDecl> {
        self.left_brace()?;
        let mut from = None;
        let mut to = None;
        let mut outside = None;
        while !self.at_right_brace() && !self.at_eof() {
            let field = self.identifier("linear mapping field")?;
            if field.value == "outside" {
                let value = super::mapping_policy::value(self);
                self.semicolon();
                assign(self, "mapping outside", &mut outside, value, field.span);
                continue;
            }
            let value = self.number("source time");
            self.semicolon();
            match field.value.as_str() {
                "from" => assign(self, "mapping from", &mut from, value, field.span),
                "to" => assign(self, "mapping to", &mut to, value, field.span),
                _ => self.mapping_field_error(&field.value, field.span),
            }
        }
        let end = self.right_brace()?;
        let span = start.join(end);
        Some(MappingDecl::Linear {
            from: mapping_required(self, "from", from, span)?,
            to: mapping_required(self, "to", to, span)?,
            outside: outside.unwrap_or_default(),
            span,
        })
    }

    fn freeze_mapping(&mut self, start: Span) -> Option<MappingDecl> {
        self.left_brace()?;
        let mut source = None;
        while !self.at_right_brace() && !self.at_eof() {
            let field = self.identifier("freeze mapping field")?;
            if field.value == "outside" {
                self.mapping_field_error(&field.value, field.span);
                self.identifier("unsupported freeze mapping field");
                self.semicolon();
                continue;
            }
            let value = self.number("freeze source time");
            self.semicolon();
            if field.value == "source" {
                assign(self, "mapping source", &mut source, value, field.span);
            } else {
                self.mapping_field_error(&field.value, field.span);
            }
        }
        let end = self.right_brace()?;
        let span = start.join(end);
        Some(MappingDecl::Freeze {
            source: mapping_required(self, "source", source, span)?,
            span,
        })
    }

    fn curve_mapping(&mut self, start: Span) -> Option<MappingDecl> {
        self.left_brace()?;
        let mut keys = Vec::new();
        let mut outside = None;
        while !self.at_right_brace() && !self.at_eof() {
            if self.at_word("key") {
                if let Some(key) = self.mapping_key() {
                    keys.push(key);
                }
            } else if self.at_word("outside") {
                let field = self.identifier("curve mapping field")?;
                let value = super::mapping_policy::value(self);
                self.semicolon();
                assign(self, "mapping outside", &mut outside, value, field.span);
            } else {
                let span = self.current().span;
                self.mapping_field_error("curve member", span);
                self.advance();
                self.recover_declaration();
            }
        }
        let end = self.right_brace()?;
        if keys.is_empty() {
            self.error(
                "AUTHORING_MAPPING_KEYS",
                "curve mapping requires at least one key".to_owned(),
                start.join(end),
            );
        }
        Some(MappingDecl::Curve {
            keys,
            outside: outside.unwrap_or_default(),
            span: start.join(end),
        })
    }

    fn mapping_key(&mut self) -> Option<MappingKey> {
        let start = self.required_word("key")?;
        let id = self.stable_identifier("mapping key")?;
        self.left_brace()?;
        let mut at = None;
        let mut source = None;
        let mut interpolation = None;
        while !self.at_right_brace() && !self.at_eof() {
            let field = self.identifier("mapping key field")?;
            match field.value.as_str() {
                "at" => {
                    let value = self.number("key record time");
                    self.semicolon();
                    assign(self, "key at", &mut at, value, field.span);
                }
                "source" => {
                    let value = self.number("key source time");
                    self.semicolon();
                    assign(self, "key source", &mut source, value, field.span);
                }
                "interpolation" => {
                    let value = self
                        .identifier("interpolation")
                        .map(|name| super::interpolation::from_identifier(self, name));
                    self.semicolon();
                    assign(
                        self,
                        "key interpolation",
                        &mut interpolation,
                        value,
                        field.span,
                    );
                }
                _ => {
                    self.mapping_field_error(&field.value, field.span);
                    self.recover_declaration();
                }
            }
        }
        let end = self.right_brace()?;
        let span = start.join(end);
        Some(MappingKey {
            id,
            at: mapping_required(self, "key at", at, span)?,
            source: mapping_required(self, "key source", source, span)?,
            interpolation: mapping_required(self, "key interpolation", interpolation, span)?,
            span,
        })
    }

    fn mapping_field_error(&mut self, field: &str, span: Span) {
        self.error(
            "AUTHORING_MAPPING_FIELD",
            format!("unsupported mapping field `{field}`"),
            span,
        );
    }
}

fn mapping_required<T>(
    parser: &mut Parser,
    name: &'static str,
    value: Option<T>,
    span: Span,
) -> Option<T> {
    if value.is_none() {
        parser.error(
            "AUTHORING_REQUIRED_FIELD",
            format!("mapping requires {name}"),
            span,
        );
    }
    value
}
