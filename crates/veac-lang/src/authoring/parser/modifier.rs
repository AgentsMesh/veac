use super::parameter::{point, scalar, vector};
use super::semantic::{boolean, finish, nested, number, required, take, word};
use super::Parser;
use crate::authoring::{
    CompositeModifierDecl, FrameDecl, Identifier, LayoutModifierDecl, ModifierDecl, PlacementDecl,
    SemanticBlock, SemanticValue, TransformModifierDecl,
};

impl Parser {
    pub(super) fn modifier(
        &mut self,
        kind: Identifier,
        id: Identifier,
        body: SemanticBlock,
    ) -> Option<ModifierDecl> {
        match kind.value.as_str() {
            "layout" => self.layout_modifier(id, body).map(ModifierDecl::Layout),
            "transform" => self
                .transform_modifier(id, body)
                .map(ModifierDecl::Transform),
            "composite" => self
                .composite_modifier(id, body)
                .map(ModifierDecl::Composite),
            "mask" => {
                let span = id.span.join(body.span);
                super::modifier_mask::parse(self, id, body, span).map(ModifierDecl::Mask)
            }
            "audio" => {
                let span = id.span.join(body.span);
                super::modifier_audio::parse(self, id, body, span).map(ModifierDecl::Audio)
            }
            "color" => {
                let span = id.span.join(body.span);
                super::modifier_color::parse(self, id, body, span).map(ModifierDecl::Color)
            }
            "effect" => self.effect_modifier(id, body).map(ModifierDecl::Effect),
            _ => {
                self.error(
                    "AUTHORING_MODIFIER_KIND",
                    "modifier must be layout, transform, composite, mask, audio, color, or effect"
                        .to_owned(),
                    kind.span,
                );
                None
            }
        }
    }

    fn layout_modifier(
        &mut self,
        id: Identifier,
        mut body: SemanticBlock,
    ) -> Option<LayoutModifierDecl> {
        let placement = take(self, &mut body, "placement").and_then(|entry| self.placement(&entry));
        let frame = take(self, &mut body, "frame").and_then(|entry| self.frame(&entry));
        let span = id.span.join(body.span);
        finish(self, body, "layout modifier");
        Some(LayoutModifierDecl {
            id,
            placement,
            frame,
            span,
        })
    }

    fn placement(&mut self, entry: &crate::authoring::SemanticEntry) -> Option<PlacementDecl> {
        let kind = match entry.values.as_slice() {
            [SemanticValue::Identifier(value)] => value.clone(),
            _ => {
                self.error(
                    "AUTHORING_PLACEMENT",
                    "placement requires anchor or absolute".to_owned(),
                    entry.span,
                );
                return None;
            }
        };
        let mut body = entry.block.clone()?;
        let value = match kind.value.as_str() {
            "anchor" => {
                let at = required(self, &mut body, "at", "anchor placement")?;
                let inset = required(self, &mut body, "inset", "anchor placement")?;
                PlacementDecl::Anchor {
                    anchor: word(self, &at, "placement at")?,
                    inset: super::modifier_static::vector(self, &inset)?,
                    span: entry.span,
                }
            }
            "absolute" => {
                let position = required(self, &mut body, "position", "absolute placement")?;
                PlacementDecl::Absolute {
                    position: super::modifier_static::point(self, &position)?,
                    span: entry.span,
                }
            }
            _ => {
                self.error(
                    "AUTHORING_PLACEMENT",
                    "placement requires anchor or absolute".to_owned(),
                    kind.span,
                );
                return None;
            }
        };
        finish(self, body, "placement");
        Some(value)
    }

    fn frame(&mut self, entry: &crate::authoring::SemanticEntry) -> Option<FrameDecl> {
        let mut body = nested(self, entry, "frame")?;
        let width = required(self, &mut body, "width", "frame")?;
        let height = required(self, &mut body, "height", "frame")?;
        let fit = required(self, &mut body, "fit", "frame")?;
        let value = FrameDecl {
            width: number(self, &width, "frame width")?,
            height: number(self, &height, "frame height")?,
            fit: word(self, &fit, "frame fit")?,
            span: entry.span,
        };
        finish(self, body, "frame");
        Some(value)
    }

    fn transform_modifier(
        &mut self,
        id: Identifier,
        mut body: SemanticBlock,
    ) -> Option<TransformModifierDecl> {
        let position = take(self, &mut body, "position").and_then(|entry| point(self, &entry));
        let scale = take(self, &mut body, "scale").and_then(|entry| vector(self, &entry));
        let rotation = take(self, &mut body, "rotation").and_then(|entry| scalar(self, &entry));
        let anchor = take(self, &mut body, "anchor")
            .and_then(|entry| super::modifier_static::vector(self, &entry));
        let crop =
            take(self, &mut body, "crop").and_then(|entry| super::parameter::rect(self, &entry));
        let flip_horizontal = take(self, &mut body, "flip-horizontal")
            .and_then(|entry| boolean(self, &entry, "flip-horizontal"));
        let flip_vertical = take(self, &mut body, "flip-vertical")
            .and_then(|entry| boolean(self, &entry, "flip-vertical"));
        let span = id.span.join(body.span);
        finish(self, body, "transform modifier");
        Some(TransformModifierDecl {
            id,
            position,
            scale,
            rotation,
            anchor,
            crop,
            flip_horizontal,
            flip_vertical,
            span,
        })
    }

    fn composite_modifier(
        &mut self,
        id: Identifier,
        mut body: SemanticBlock,
    ) -> Option<CompositeModifierDecl> {
        let opacity = take(self, &mut body, "opacity").and_then(|entry| scalar(self, &entry));
        let z_index =
            take(self, &mut body, "z-index").and_then(|entry| number(self, &entry, "z-index"));
        let blend = take(self, &mut body, "blend").and_then(|entry| word(self, &entry, "blend"));
        let span = id.span.join(body.span);
        finish(self, body, "composite modifier");
        Some(CompositeModifierDecl {
            id,
            opacity,
            z_index,
            blend,
            span,
        })
    }
}
