use crate::authoring::SidechainRelation;
use veac_ir::RelationKind;

use super::context::Context;
use super::{relation, value};

pub(super) fn lower(
    ctx: &mut Context,
    declaration: &SidechainRelation,
    sequence: &veac_ir::Sequence,
) -> Option<RelationKind> {
    let key = relation::signal_endpoint(ctx, &declaration.endpoints.key)?;
    let target = relation::item_endpoint(ctx, &declaration.endpoints.target)?;
    let target_id = relation::item_id(ctx, &declaration.endpoints.target)?;
    let threshold_db = value::scalar(ctx, &declaration.dynamics.threshold, "db")?;
    let ratio = value::unitless(ctx, &declaration.dynamics.ratio)?;
    let attack_ms = value::milliseconds(ctx, &declaration.dynamics.attack)?;
    let release_ms = value::milliseconds(ctx, &declaration.dynamics.release)?;
    let Some(clip) = relation::item(sequence, &target_id) else {
        ctx.error(
            "AUTHORING_LOWER_RELATION_ITEM",
            "sidechain target item does not exist in its sequence",
            declaration.endpoints.target.id.span,
        );
        return None;
    };
    let active_range = match &declaration.timing.active {
        Some(active) => Some(local_range(ctx, active, clip.record_range)?),
        None => None,
    };
    if clip.audio.is_none() {
        ctx.error(
            "AUTHORING_LOWER_RELATION_COMPONENT",
            "sidechain target must have an audio component",
            declaration.endpoints.target.id.span,
        );
        return None;
    };
    let parameters = veac_ir::SidechainRelationParameters {
        threshold_db,
        ratio,
        attack_ms,
        release_ms,
        active_range,
    };
    Some(RelationKind::Sidechain {
        key,
        target,
        parameters,
    })
}

fn local_range(
    ctx: &mut Context,
    value: &crate::authoring::RecordSpan,
    target: veac_ir::TimeRange,
) -> Option<veac_ir::TimeRange> {
    let record = value::range(ctx, &value.at, &value.duration)?;
    let Some(offset) = record.start.value.checked_sub(target.start.value) else {
        ctx.error(
            "AUTHORING_LOWER_RELATION_RANGE",
            "sidechain active range cannot start before its target item",
            value.at.span,
        );
        return None;
    };
    let start = veac_ir::RationalTime::new(offset, record.start.timescale).ok()?;
    veac_ir::TimeRange::new(start, record.duration).ok()
}
