use crate::authoring::{ItemEndpoint, RelationDecl, RelationKind, SequenceDecl, SignalEndpoint};
use veac_ir::{Clip, ItemId, Relation, RelationEndpoint, Sequence};

use super::context::Context;
use super::{ids, relation_membership, relation_sidechain, relation_visual};

pub(super) fn lower_sequence(
    ctx: &mut Context,
    declaration: &SequenceDecl,
    sequence: &Sequence,
) -> Vec<Relation> {
    declaration
        .structures
        .iter()
        .filter_map(|structure| match structure {
            crate::authoring::StructureDecl::Relation(value) => lower(ctx, value, sequence),
            _ => None,
        })
        .collect()
}

fn lower(ctx: &mut Context, declaration: &RelationDecl, sequence: &Sequence) -> Option<Relation> {
    let kind = match &declaration.kind {
        RelationKind::Transition(value) => relation_visual::transition(ctx, value)?,
        RelationKind::Matte(value) => relation_visual::matte(ctx, value, sequence)?,
        RelationKind::Sidechain(value) => relation_sidechain::lower(ctx, value, sequence)?,
        RelationKind::Group(value) => relation_membership::group(ctx, &value.members)?,
        RelationKind::AvLink(value) => relation_membership::av_link(ctx, value)?,
    };
    Some(Relation {
        id: ids::relation(ctx, &declaration.id)?,
        sequence_id: sequence.id.clone(),
        kind,
    })
}

pub(super) fn item_endpoint(ctx: &mut Context, value: &ItemEndpoint) -> Option<RelationEndpoint> {
    Some(RelationEndpoint::item(ids::item(ctx, &value.id)?))
}

pub(super) fn signal_endpoint(
    ctx: &mut Context,
    value: &SignalEndpoint,
) -> Option<RelationEndpoint> {
    Some(match value {
        SignalEndpoint::Track { id } => RelationEndpoint::track(ids::track(ctx, id)?),
        SignalEndpoint::Bus { id } => RelationEndpoint::bus(ids::bus_id(ctx, id)?),
    })
}

pub(super) fn item_id(ctx: &mut Context, value: &ItemEndpoint) -> Option<ItemId> {
    ids::item(ctx, &value.id)
}

pub(super) fn item<'a>(sequence: &'a Sequence, id: &ItemId) -> Option<&'a Clip> {
    sequence
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .find(|item| &item.id == id)
}
