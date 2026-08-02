use crate::authoring::{ModifierDecl, Span};
use veac_ir::{AudioProperties, EffectInstance, VisualProperties};

use super::context::Context;

pub(super) struct LoweredModifiers {
    pub visual: VisualProperties,
    pub effects: Vec<EffectInstance>,
    pub audio: Option<AudioProperties>,
}

pub(super) fn lower(
    context: &mut Context,
    values: &[ModifierDecl],
    default_z_index: i32,
) -> Option<LoweredModifiers> {
    let mut result = LoweredModifiers {
        visual: super::visual_default::value(default_z_index),
        effects: Vec::new(),
        audio: None,
    };
    let mut layout = None;
    let mut transform = None;
    let mut composite = None;
    let mut surface = None;
    let mut audio = None;
    let mut color = None;
    for value in values {
        match value {
            ModifierDecl::Layout(value) => {
                unique(context, &mut layout, value.span, "layout")?;
                super::modifier_visual::layout(context, &mut result.visual, value)?;
            }
            ModifierDecl::Transform(value) => {
                unique(context, &mut transform, value.span, "transform")?;
                super::modifier_visual::transform(context, &mut result.visual, value)?;
            }
            ModifierDecl::Composite(value) => {
                unique(context, &mut composite, value.span, "composite")?;
                super::modifier_visual::composite(context, &mut result.visual, value)?;
            }
            ModifierDecl::Surface(value) => {
                unique(context, &mut surface, value.span, "surface")?;
                result.visual.card = Some(super::modifier_surface::lower(context, value)?);
            }
            ModifierDecl::Mask(value) => {
                result
                    .visual
                    .masks
                    .push(super::modifier_mask::lower(context, value)?);
            }
            ModifierDecl::Audio(value) => {
                unique(context, &mut audio, value.span, "audio")?;
                result.audio = Some(super::modifier_audio::lower(context, value)?);
            }
            ModifierDecl::Color(value) => {
                unique(context, &mut color, value.span, "color")?;
                result.visual.color_pipeline = Some(super::modifier_color::lower(context, value)?);
            }
            ModifierDecl::Effect(value) => {
                result
                    .effects
                    .push(super::modifier_effect::lower(context, value)?);
            }
        }
    }
    Some(result)
}

fn unique(
    context: &mut Context,
    previous: &mut Option<Span>,
    span: Span,
    kind: &'static str,
) -> Option<()> {
    if previous.replace(span).is_some() {
        context.error(
            "AUTHORING_LOWER_DUPLICATE_MODIFIER",
            format!("an item may contain only one {kind} modifier"),
            span,
        );
        None
    } else {
        Some(())
    }
}
