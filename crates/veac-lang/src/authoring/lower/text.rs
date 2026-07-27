use crate::authoring::{CaptionSourceDecl, TextSourceDecl};
use veac_ir::ClipSource;

use super::context::Context;

pub(super) fn lower(ctx: &mut Context, value: &TextSourceDecl) -> Option<ClipSource> {
    Some(ClipSource::Text {
        text: value.content.value.clone(),
        style: style(ctx, value)?,
    })
}

pub(super) fn caption(ctx: &mut Context, value: &CaptionSourceDecl) -> Option<ClipSource> {
    Some(ClipSource::Caption {
        text: value.text.content.value.clone(),
        speaker: value.speaker.as_ref().map(|value| value.value.clone()),
        style: style(ctx, &value.text)?,
    })
}

fn style(ctx: &mut Context, value: &TextSourceDecl) -> Option<veac_ir::TextStyle> {
    let style = super::text_style::lower(ctx, &value.style)?;
    let (layout, path) = super::text_layout::lower(ctx, &value.layout)?;
    let animation = match &value.animation {
        Some(value) => Some(super::text_animation::lower(ctx, value)?),
        None => None,
    };
    Some(veac_ir::TextStyle {
        layout,
        path,
        animation,
        ..style
    })
}
