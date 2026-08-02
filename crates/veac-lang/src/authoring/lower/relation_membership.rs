use crate::authoring::{AvLinkRelation, ItemEndpoint};
use veac_ir::RelationKind;

use super::context::Context;
use super::relation;

pub(super) fn group(ctx: &mut Context, members: &[ItemEndpoint]) -> Option<RelationKind> {
    let members = members
        .iter()
        .map(|value| relation::item_endpoint(ctx, value))
        .collect::<Option<Vec<_>>>()?;
    Some(RelationKind::Group { members })
}

pub(super) fn av_link(ctx: &mut Context, declaration: &AvLinkRelation) -> Option<RelationKind> {
    let video = relation::item_endpoint(ctx, &declaration.video)?;
    let audio = declaration
        .audio
        .iter()
        .map(|value| relation::item_endpoint(ctx, value))
        .collect::<Option<Vec<_>>>()?;
    Some(RelationKind::AvLink { video, audio })
}
