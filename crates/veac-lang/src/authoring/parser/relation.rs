use crate::authoring::{Diagnostic, RelationDecl, RelationKind, SemanticBlock};

use super::Parser;

impl Parser {
    pub(super) fn relation(&mut self) -> Option<RelationDecl> {
        let start = self.required_word("relation")?;
        let relation_type = self.identifier("relation type")?;
        let id = self.stable_identifier("relation id")?;
        let body = self.semantic_block()?;
        let span = start.join(body.span);
        let kind = match relation_type.value.as_str() {
            "transition" => {
                super::relation_transition::parse(self, body).map(RelationKind::Transition)
            }
            "matte" => super::relation_matte::parse(self, body).map(RelationKind::Matte),
            "sidechain" => {
                super::relation_sidechain::parse(self, body).map(RelationKind::Sidechain)
            }
            "group" => super::relation_membership::group(self, body).map(RelationKind::Group),
            "av-link" => super::relation_membership::av_link(self, body).map(RelationKind::AvLink),
            _ => {
                self.diagnostic(Diagnostic::new(
                    "AUTHORING_UNKNOWN_RELATION_KIND",
                    format!("unknown relation kind {}", relation_type.value),
                    relation_type.span,
                ));
                None
            }
        }?;
        Some(RelationDecl { id, kind, span })
    }
}

pub(super) fn required_block(
    parser: &mut Parser,
    body: &mut SemanticBlock,
    name: &str,
    context: &str,
) -> Option<SemanticBlock> {
    super::relation_value::take_block(parser, body, name, true, context)
}
