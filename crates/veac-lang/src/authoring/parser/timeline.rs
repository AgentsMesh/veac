use super::item::assign;
use super::Parser;
use crate::authoring::{LayerDecl, LayerKind, SequenceDecl, StructureDecl};

impl Parser {
    pub(super) fn sequence(&mut self) -> Option<SequenceDecl> {
        let start = self.required_word("sequence")?;
        let id = self.identifier("sequence")?;
        self.left_brace()?;
        let mut layers = Vec::new();
        let mut structures = Vec::new();
        while !self.at_right_brace() && !self.at_eof() {
            if self.at_word("layer") {
                if let Some(layer) = self.layer() {
                    layers.push(layer);
                }
            } else if self.at_word("relation") {
                if let Some(value) = self.relation() {
                    structures.push(StructureDecl::Relation(value));
                }
            } else if self.at_word("apply") {
                if let Some(value) = self.apply() {
                    structures.push(StructureDecl::Apply(value));
                }
            } else {
                let span = self.current().span;
                self.error(
                    "AUTHORING_SEQUENCE_MEMBER",
                    "sequence member must be layer, relation, or apply".to_owned(),
                    span,
                );
                self.advance();
                self.recover_declaration();
            }
        }
        let end = self.right_brace()?;
        Some(SequenceDecl {
            id,
            layers,
            structures,
            span: start.join(end),
        })
    }

    fn layer(&mut self) -> Option<LayerDecl> {
        let start = self.required_word("layer")?;
        let kind = self.identifier("layer type")?;
        let kind = match kind.value.as_str() {
            "video" => LayerKind::Video,
            "audio" => LayerKind::Audio,
            "visual" => LayerKind::Visual,
            "caption" => LayerKind::Caption,
            _ => {
                self.error(
                    "AUTHORING_LAYER_TYPE",
                    "layer type must be video, audio, visual, or caption".to_owned(),
                    kind.span,
                );
                return None;
            }
        };
        let id = self.identifier("layer")?;
        self.left_brace()?;
        let mut placement = None;
        let mut state = None;
        let mut order = None;
        let mut route_bus = None;
        let mut items = Vec::new();
        while !self.at_right_brace() && !self.at_eof() {
            if self.at_word("placement") {
                let field = self.required_word("placement")?;
                let value = self
                    .identifier("placement mode")
                    .and_then(|value| super::timeline_state::placement(self, value));
                self.semicolon();
                assign(self, "layer placement", &mut placement, value, field);
            } else if self.at_word("state") {
                let field = self.required_word("state")?;
                let value = self
                    .semantic_block()
                    .and_then(|block| super::timeline_state::track(self, block, kind));
                assign(self, "layer state", &mut state, value, field);
            } else if self.at_word("order") {
                let field = self.advance().span;
                let value = self.number("layer order");
                self.semicolon();
                if order.is_some() {
                    self.duplicate("layer order", field);
                } else {
                    order = value;
                }
            } else if self.at_word("route") {
                let field = self.advance().span;
                self.required_word("bus");
                let value = self.identifier("audio bus");
                self.semicolon();
                if route_bus.is_some() {
                    self.duplicate("route bus", field);
                } else {
                    route_bus = value;
                }
            } else if self.at_word("item") {
                if let Some(item) = self.item() {
                    items.push(item);
                }
            } else {
                let span = self.current().span;
                self.error(
                    "AUTHORING_LAYER_MEMBER",
                    "layer members must be placement, state, order, route, or item".to_owned(),
                    span,
                );
                self.advance();
                self.recover_declaration();
            }
        }
        let end = self.right_brace()?;
        if route_bus.is_some() && kind != LayerKind::Audio {
            self.error(
                "AUTHORING_LAYER_ROUTE",
                "only audio layers can route to a bus".to_owned(),
                start.join(end),
            );
        }
        Some(LayerDecl {
            kind,
            id,
            placement,
            state,
            order,
            route_bus,
            items,
            span: start.join(end),
        })
    }
}
