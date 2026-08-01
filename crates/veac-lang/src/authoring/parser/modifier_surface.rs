use crate::authoring::{Identifier, SemanticBlock, SurfaceModifierDecl};

use super::{semantic, Parser};

pub(super) fn parse(
    parser: &mut Parser,
    id: Identifier,
    mut body: SemanticBlock,
) -> Option<SurfaceModifierDecl> {
    let corner_entry = semantic::required(parser, &mut body, "corner-radius", "surface modifier")?;
    let corner_radius = semantic::number(parser, &corner_entry, "surface corner-radius")?;
    let shadow = semantic::take(parser, &mut body, "shadow")
        .and_then(|entry| super::shadow::parse(parser, &entry, "surface shadow"));
    let span = id.span.join(body.span);
    semantic::finish(parser, body, "surface modifier");
    Some(SurfaceModifierDecl {
        id,
        corner_radius,
        shadow,
        span,
    })
}
