use std::collections::BTreeSet;

use crate::authoring::{ApplyItemTarget, ApplyScope, Identifier, Span};
use veac_ir::{ApplyTarget, ItemId, Relation, RelationGraph, Sequence, Track, TrackKind};

use super::context::Context;
use super::ids;

pub(super) fn resolve(
    ctx: &mut Context,
    scope: &ApplyScope,
    sequence: &Sequence,
    relations: &[Relation],
) -> Option<ApplyTarget> {
    match scope {
        ApplyScope::CompositeBand {
            from,
            through,
            span,
        } => composite_band(ctx, sequence, from, through, *span),
        ApplyScope::Layer { layer, .. } => Some(ApplyTarget::Layer {
            track_id: visual_track(ctx, sequence, layer)?.id.clone(),
        }),
        ApplyScope::Items { targets, .. } => item_set(ctx, sequence, relations, targets),
    }
}

fn composite_band(
    ctx: &mut Context,
    sequence: &Sequence,
    from: &Identifier,
    through: &Identifier,
    span: Span,
) -> Option<ApplyTarget> {
    let from = visual_track(ctx, sequence, from)?;
    let through = visual_track(ctx, sequence, through)?;
    if from.order > through.order {
        ctx.error(
            "AUTHORING_LOWER_APPLY_BAND_ORDER",
            "composite-band from layer must not be above its through layer",
            span,
        );
        return None;
    }
    Some(ApplyTarget::CompositeBand {
        from_track_id: from.id.clone(),
        through_track_id: through.id.clone(),
    })
}

fn item_set(
    ctx: &mut Context,
    sequence: &Sequence,
    relations: &[Relation],
    targets: &[ApplyItemTarget],
) -> Option<ApplyTarget> {
    let graph = RelationGraph::scoped(sequence, relations);
    let mut items = BTreeSet::new();
    let mut valid = true;
    for target in targets {
        let resolved = match target {
            ApplyItemTarget::Item(value) => graph
                .item(&sequence.id, &ids::item(ctx, value)?)
                .map(|item| vec![item])
                .or_else(|| missing(ctx, value, "item in the owning sequence")),
            ApplyItemTarget::Group(value) => {
                let id = ids::relation(ctx, value)?;
                graph
                    .groups(&sequence.id)
                    .into_iter()
                    .find(|edge| edge.relation_id == &id)
                    .map(|edge| edge.members)
                    .or_else(|| missing(ctx, value, "group in the owning sequence"))
            }
        }?;
        for member in resolved {
            if !visual(member.track.kind) {
                ctx.error(
                    "AUTHORING_LOWER_APPLY_NON_VISUAL_ITEM",
                    "apply items scope accepts only items on visual layers",
                    target.id().span,
                );
                valid = false;
            } else if !items.insert(member.clip.id.clone()) {
                ctx.error(
                    "AUTHORING_LOWER_APPLY_DUPLICATE_ITEM",
                    "apply items scope resolves the same item more than once",
                    target.id().span,
                );
                valid = false;
            }
        }
    }
    valid.then_some(ApplyTarget::ItemSet {
        item_ids: items.into_iter().collect::<Vec<ItemId>>(),
    })
}

fn visual_track<'a>(
    ctx: &mut Context,
    sequence: &'a Sequence,
    id: &Identifier,
) -> Option<&'a Track> {
    let target = ids::track(ctx, id)?;
    sequence
        .tracks
        .iter()
        .find(|track| track.id == target && visual(track.kind))
        .or_else(|| missing(ctx, id, "visual layer in the owning sequence"))
}

fn visual(kind: TrackKind) -> bool {
    matches!(kind, TrackKind::Video | TrackKind::Visual)
}

fn missing<T>(ctx: &mut Context, id: &Identifier, kind: &str) -> Option<T> {
    ctx.error(
        "AUTHORING_LOWER_APPLY_TARGET",
        format!("apply target '{}' is not a {kind}", id.value),
        id.span,
    );
    None
}
