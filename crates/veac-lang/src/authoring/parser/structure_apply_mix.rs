use super::{parameter, semantic, Parser};
use crate::authoring::{ApplyMixDecl, MaskModifierDecl, SemanticEntry, SemanticValue};

impl Parser {
    pub(super) fn apply_mix(&mut self) -> Option<ApplyMixDecl> {
        let start = self.required_word("mix")?;
        let mut body = self.semantic_block()?;
        let opacity = semantic::take(self, &mut body, "opacity")
            .and_then(|entry| parameter::scalar(self, &entry));
        let blend = semantic::take(self, &mut body, "blend")
            .and_then(|entry| semantic::word(self, &entry, "apply mix blend"));
        let masks = body
            .entries
            .drain(..)
            .filter_map(|entry| self.apply_mix_mask(entry))
            .collect();
        Some(ApplyMixDecl {
            opacity,
            blend,
            masks,
            span: start.join(body.span),
        })
    }

    fn apply_mix_mask(&mut self, entry: SemanticEntry) -> Option<MaskModifierDecl> {
        if entry.name.value != "mask" {
            self.error(
                "AUTHORING_APPLY_MIX_FIELD",
                "apply mix accepts only opacity, blend, and mask".to_owned(),
                entry.name.span,
            );
            return None;
        }
        let [SemanticValue::Identifier(id)] = entry.values.as_slice() else {
            semantic::shape(
                self,
                &entry,
                "mask requires one stable identifier".to_owned(),
            );
            return None;
        };
        let id = id.clone();
        let Some(body) = entry.block else {
            semantic::shape(self, &entry, "mask requires a block".to_owned());
            return None;
        };
        let span = entry.name.span.join(body.span);
        super::modifier_mask::parse(self, id, body, span)
    }
}
