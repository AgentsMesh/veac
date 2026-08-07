use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::TimeRange;

use super::{malformed, ExecutableLowerError};
use crate::program::executable::lower::{time, value};

pub(super) struct EffectState {
    pub(super) enabled: bool,
    pub(super) range: Option<TimeRange>,
}

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
) -> Result<EffectState, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::EffectEnabled, [window]) => Ok(EffectState {
            enabled: true,
            range: window_value(graph, window, timebase)?,
        }),
        (Op::EffectDisabled, [window]) => Ok(EffectState {
            enabled: false,
            range: window_value(graph, window, timebase)?,
        }),
        _ => Err(malformed()),
    }
}

fn window_value(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
) -> Result<Option<TimeRange>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::EffectWindowFull, []) => Ok(None),
        (Op::EffectWindowDuring, [range]) => Ok(Some(time::range(graph, range, timebase)?)),
        _ => Err(malformed()),
    }
}
