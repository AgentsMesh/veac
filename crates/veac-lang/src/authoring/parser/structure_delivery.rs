mod artifact;

use crate::authoring::DeliveryDecl;

use super::item::{assign, required as required_value};
use super::Parser;

impl Parser {
    pub(super) fn delivery(&mut self) -> Option<DeliveryDecl> {
        let start = self.required_word("delivery")?;
        let id = self.identifier("delivery")?;
        self.left_brace()?;
        let mut sequence = None;
        let mut raster = None;
        let mut artifacts = Vec::new();
        while !self.at_right_brace() && !self.at_eof() {
            if self.at_word("sequence") {
                let field = self.required_word("sequence")?;
                let value = self.identifier("delivery sequence");
                self.semicolon();
                assign(self, "delivery sequence", &mut sequence, value, field);
            } else if self.at_word("raster") {
                let field = self.required_word("raster")?;
                let value = self
                    .semantic_block()
                    .and_then(|block| super::delivery_raster::parse(self, block));
                assign(self, "delivery raster", &mut raster, value, field);
            } else if self.at_word("artifact") {
                if let Some(value) = self.artifact() {
                    artifacts.push(value);
                }
            } else {
                let span = self.current().span;
                self.error(
                    "AUTHORING_DELIVERY_MEMBER",
                    "delivery member must be sequence, raster, or artifact".to_owned(),
                    span,
                );
                self.advance();
                self.recover_declaration();
            }
        }
        let end = self.right_brace()?;
        let span = start.join(end);
        if artifacts.is_empty() {
            self.error(
                "AUTHORING_DELIVERY_ARTIFACT_REQUIRED",
                "delivery requires at least one artifact".to_owned(),
                span,
            );
        }
        Some(DeliveryDecl {
            id,
            sequence: required_value(self, "delivery sequence", sequence, span)?,
            raster,
            artifacts,
            span,
        })
    }
}
