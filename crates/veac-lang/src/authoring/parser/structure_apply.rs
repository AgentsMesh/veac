use super::item::{assign, required};
use super::Parser;
use crate::authoring::{ApplyDecl, ApplyStageDecl};

impl Parser {
    pub(super) fn apply(&mut self) -> Option<ApplyDecl> {
        let start = self.required_word("apply")?;
        let id = self.stable_identifier("apply")?;
        self.left_brace()?;
        let mut scope = None;
        let mut record = None;
        let mut pipeline = None;
        let mut mix = None;
        while !self.at_right_brace() && !self.at_eof() {
            if self.at_word("scope") {
                let span = self.current().span;
                let value = self.apply_scope();
                assign(self, "apply scope", &mut scope, value, span);
            } else if self.at_word("record") {
                let span = self.current().span;
                let value = self.record_span();
                assign(self, "apply record", &mut record, value, span);
            } else if self.at_word("pipeline") {
                let span = self.current().span;
                let value = self.apply_pipeline();
                assign(self, "apply pipeline", &mut pipeline, value, span);
            } else if self.at_word("mix") {
                let span = self.current().span;
                let value = self.apply_mix();
                assign(self, "apply mix", &mut mix, value, span);
            } else {
                let span = self.current().span;
                self.error(
                    "AUTHORING_APPLY_FIELD",
                    "apply field must be scope, record, pipeline, or mix".to_owned(),
                    span,
                );
                self.advance();
                self.recover_declaration();
            }
        }
        let end = self.right_brace()?;
        let span = start.join(end);
        Some(ApplyDecl {
            id,
            scope: required(self, "scope", scope, span)?,
            record: required(self, "record", record, span)?,
            pipeline: required(self, "pipeline", pipeline, span)?,
            mix: required(self, "mix", mix, span)?,
            span,
        })
    }

    fn apply_pipeline(&mut self) -> Option<Vec<ApplyStageDecl>> {
        let start = self.required_word("pipeline")?;
        self.left_brace()?;
        let mut stages = Vec::new();
        while !self.at_right_brace() && !self.at_eof() {
            if !self.at_word("stage") {
                self.error(
                    "AUTHORING_APPLY_PIPELINE_MEMBER",
                    "apply pipeline accepts only stage declarations".to_owned(),
                    self.current().span,
                );
                self.advance();
                self.recover_declaration();
                continue;
            }
            self.advance();
            let kind = self.identifier("apply stage kind")?;
            let id = self.stable_identifier("apply stage")?;
            let body = self.semantic_block()?;
            let span = kind.span.join(body.span);
            let stage = match kind.value.as_str() {
                "color" => {
                    ApplyStageDecl::Color(super::modifier_color::parse(self, id, body, span)?)
                }
                "effect" => ApplyStageDecl::Effect(self.effect_modifier(id, body)?),
                _ => {
                    self.error(
                        "AUTHORING_APPLY_STAGE_KIND",
                        "apply stage must be color or effect".to_owned(),
                        kind.span,
                    );
                    continue;
                }
            };
            stages.push(stage);
        }
        let end = self.right_brace()?;
        if stages.is_empty() {
            self.error(
                "AUTHORING_REQUIRED_FIELD",
                "apply pipeline requires at least one stage".to_owned(),
                start.join(end),
            );
        }
        Some(stages)
    }
}
