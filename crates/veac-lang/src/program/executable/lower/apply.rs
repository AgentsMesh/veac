use crate::program::expression::runtime::domain_graph::{FrozenDomainGraph, FrozenEntity};
use crate::program::expression::Value;
use crate::program::{DomainOperationId as Op, DomainType};
use veac_ir::{Apply, ApplyMix, ApplyTarget};

use super::error::ExecutableLowerError;
use super::{animation, id, time, value, visual};

mod stage;

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    entity: FrozenEntity<'_>,
    timebase: u32,
) -> Result<Apply, ExecutableLowerError> {
    let values = value::entity_operands(entity, DomainType::Apply, Op::Apply)?;
    if values.len() != 6 {
        return Err(malformed());
    }
    let path = entity.logical_path().ok_or_else(malformed)?;
    Ok(Apply {
        id: id::apply(&path),
        enabled: state(graph, &values[1])?,
        record_range: time::range(graph, &values[2], timebase)?,
        target: target(graph, &values[3])?,
        stages: value::list(values.get(4))?
            .iter()
            .map(|source| stage::lower(graph, source, timebase, &path))
            .collect::<Result<_, _>>()?,
        mix: mix(graph, &values[5], timebase, &path)?,
    })
}

fn state(graph: &FrozenDomainGraph, source: &Value) -> Result<bool, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::ApplyEnabled, []) => Ok(true),
        (Op::ApplyDisabled, []) => Ok(false),
        _ => Err(malformed()),
    }
}

fn target(graph: &FrozenDomainGraph, source: &Value) -> Result<ApplyTarget, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::ApplyTargetCompositeBand, [from, through]) => Ok(ApplyTarget::CompositeBand {
            from_track_id: track_ref(graph, from)?,
            through_track_id: track_ref(graph, through)?,
        }),
        (Op::ApplyTargetLayer, [layer]) => Ok(ApplyTarget::Layer {
            track_id: track_ref(graph, layer)?,
        }),
        (Op::ApplyTargetItems, [items]) => {
            let mut item_ids = value::list(Some(items))?
                .iter()
                .map(|item| item_ref(graph, item))
                .collect::<Result<Vec<_>, _>>()?;
            item_ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
            Ok(ApplyTarget::ItemSet { item_ids })
        }
        _ => Err(malformed()),
    }
}

fn mix(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    scope: &[&str],
) -> Result<ApplyMix, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::ApplyMix)?;
    let [opacity, blend, masks] = values else {
        return Err(malformed());
    };
    let mut mix_scope = scope.to_vec();
    mix_scope.push("mix");
    let mut opacity_scope = mix_scope.clone();
    opacity_scope.push("opacity");
    Ok(ApplyMix {
        opacity: animation::percent(graph, opacity, timebase, &opacity_scope)?,
        blend_mode: visual::blend(graph, blend)?,
        masks: visual::masks(graph, value::list(Some(masks))?, timebase, &mix_scope)?,
    })
}

pub(super) fn item_ref(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<veac_ir::ItemId, ExecutableLowerError> {
    Ok(id::item(&logical_ref(graph, source)?))
}

pub(super) fn track_ref(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<veac_ir::TrackId, ExecutableLowerError> {
    Ok(id::track(&logical_ref(graph, source)?))
}

pub(super) fn apply_ref(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<veac_ir::ApplyId, ExecutableLowerError> {
    Ok(id::apply(&logical_ref(graph, source)?))
}

fn logical_ref<'a>(
    graph: &'a FrozenDomainGraph,
    source: &Value,
) -> Result<Vec<&'a str>, ExecutableLowerError> {
    let Value::Domain(handle) = source else {
        return Err(malformed());
    };
    graph.logical_key(handle).ok_or_else(malformed)
}

pub(super) fn malformed() -> ExecutableLowerError {
    value::graph("an executable adjustment apply has invalid typed topology")
}
