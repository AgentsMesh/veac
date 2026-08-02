use crate::authoring::ShadowDecl;
use veac_ir::{Shadow, Vec2};

use super::context::Context;
use super::{color, value};

pub(super) fn lower(ctx: &mut Context, declaration: &ShadowDecl) -> Option<Shadow> {
    Some(Shadow {
        blur_pixels: value::scalar(ctx, &declaration.blur, "px")?,
        opacity: value::scale(ctx, &declaration.opacity)?,
        offset: Vec2 {
            x: value::scalar(ctx, &declaration.offset.x, "px")?,
            y: value::scalar(ctx, &declaration.offset.y, "px")?,
        },
        color: color::lower(ctx, &declaration.color)?,
    })
}
