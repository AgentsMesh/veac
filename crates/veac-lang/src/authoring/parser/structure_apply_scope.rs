use super::item::{assign, required};
use super::Parser;
use crate::authoring::{ApplyItemTarget, ApplyScope};

impl Parser {
    pub(super) fn apply_scope(&mut self) -> Option<ApplyScope> {
        let start = self.required_word("scope")?;
        if self.at_left_brace() {
            self.error(
                "AUTHORING_APPLY_SCOPE_KIND",
                "scope must name composite-band, layer, or items".to_owned(),
                self.current().span,
            );
            self.recover_declaration();
            return None;
        }
        let kind = self.identifier("apply scope kind")?;
        match kind.value.as_str() {
            "composite-band" => self.composite_band_scope(start),
            "layer" => {
                let layer = self.identifier("apply layer")?;
                let end = self.semicolon()?;
                Some(ApplyScope::Layer {
                    layer,
                    span: start.join(end),
                })
            }
            "items" => self.item_set_scope(start),
            _ => {
                self.error(
                    "AUTHORING_APPLY_SCOPE_KIND",
                    "scope must be composite-band, layer, or items".to_owned(),
                    kind.span,
                );
                self.recover_declaration();
                None
            }
        }
    }

    fn composite_band_scope(&mut self, start: crate::authoring::Span) -> Option<ApplyScope> {
        self.left_brace()?;
        let mut from = None;
        let mut through = None;
        while !self.at_right_brace() && !self.at_eof() {
            let field = self.identifier("composite band boundary")?;
            self.required_word("layer")?;
            let layer = self.identifier("composite band layer")?;
            self.semicolon()?;
            match field.value.as_str() {
                "from" => assign(self, "band from", &mut from, Some(layer), field.span),
                "through" => assign(self, "band through", &mut through, Some(layer), field.span),
                _ => self.error(
                    "AUTHORING_APPLY_BAND_FIELD",
                    "composite-band accepts only from and through".to_owned(),
                    field.span,
                ),
            }
        }
        let end = self.right_brace()?;
        let span = start.join(end);
        Some(ApplyScope::CompositeBand {
            from: required(self, "band from", from, span)?,
            through: required(self, "band through", through, span)?,
            span,
        })
    }

    fn item_set_scope(&mut self, start: crate::authoring::Span) -> Option<ApplyScope> {
        self.left_brace()?;
        let mut targets = Vec::new();
        while !self.at_right_brace() && !self.at_eof() {
            let reference = self.typed_reference()?;
            self.semicolon()?;
            match reference.kind.value.as_str() {
                "item" => targets.push(ApplyItemTarget::Item(reference.id)),
                "group" => targets.push(ApplyItemTarget::Group(reference.id)),
                _ => self.error(
                    "AUTHORING_APPLY_ITEM_TARGET",
                    "items scope accepts only item or group targets".to_owned(),
                    reference.kind.span,
                ),
            }
        }
        let end = self.right_brace()?;
        if targets.is_empty() {
            self.error(
                "AUTHORING_REQUIRED_FIELD",
                "items scope requires at least one target".to_owned(),
                start.join(end),
            );
        }
        Some(ApplyScope::Items {
            targets,
            span: start.join(end),
        })
    }
}
