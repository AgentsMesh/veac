use crate::authoring::SurfaceModifierDecl;
use veac_ir::CardStyle;

use super::context::Context;
use super::value;

pub(super) fn lower(ctx: &mut Context, declaration: &SurfaceModifierDecl) -> Option<CardStyle> {
    Some(CardStyle {
        corner_radius_pixels: value::scalar(ctx, &declaration.corner_radius, "px")?,
        shadow: match &declaration.shadow {
            Some(value) => Some(super::shadow::lower(ctx, value)?),
            None => None,
        },
    })
}
