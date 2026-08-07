use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{ApplyOperation, ApplyStage};

use super::{malformed, ExecutableLowerError};
use crate::program::executable::lower::{color, effect, id, time, value};

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    scope: &[&str],
) -> Result<ApplyStage, ExecutableLowerError> {
    let (operation, values) = value::description(graph, source)?;
    let [key, state, payload] = values else {
        return Err(malformed());
    };
    let key = value::identifier(Some(key))?;
    let mut path = scope.to_vec();
    path.push(key);
    let (enabled, active_range) = stage_state(graph, state, timebase)?;
    let operation = match operation {
        Op::ApplyColorStage => ApplyOperation::Color {
            pipeline: color::pipeline(graph, payload)?,
        },
        Op::ApplyEffectStage => ApplyOperation::Effect {
            effect: effect::description(graph, payload, timebase, &path)?,
        },
        _ => return Err(malformed()),
    };
    Ok(ApplyStage {
        id: id::apply_stage(&path),
        enabled,
        active_range,
        operation,
    })
}

fn stage_state(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
) -> Result<(bool, Option<veac_ir::TimeRange>), ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::ApplyStageEnabled, [window]) => Ok((true, window_value(graph, window, timebase)?)),
        (Op::ApplyStageDisabled, [window]) => Ok((false, window_value(graph, window, timebase)?)),
        _ => Err(malformed()),
    }
}

fn window_value(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
) -> Result<Option<veac_ir::TimeRange>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::ApplyStageWindowFull, []) => Ok(None),
        (Op::ApplyStageWindowDuring, [range]) => Ok(Some(time::range(graph, range, timebase)?)),
        _ => Err(malformed()),
    }
}
