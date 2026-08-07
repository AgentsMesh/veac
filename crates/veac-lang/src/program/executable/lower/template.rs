use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{FillMode, SlotConstraint, SlotKind};

use super::error::ExecutableLowerError;
use super::{time, value};

pub(super) struct LoweredTemplate {
    pub(super) constraint: SlotConstraint,
    pub(super) editable_text: bool,
}

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
) -> Result<LoweredTemplate, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::TemplateContract)?;
    if values.len() != 5 {
        return Err(malformed());
    }
    Ok(LoweredTemplate {
        constraint: SlotConstraint {
            kind: slot_kind(graph, &values[0])?,
            fill: fill(graph, &values[1])?,
            label: value::text(values.get(2))?.to_owned(),
            min_source_duration: duration(graph, &values[3], timebase)?,
        },
        editable_text: text_policy(graph, &values[4])?,
    })
}

fn slot_kind(graph: &FrozenDomainGraph, source: &Value) -> Result<SlotKind, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::SlotVideo, []) => Ok(SlotKind::Video),
        (Op::SlotImage, []) => Ok(SlotKind::Image),
        (Op::SlotVideoOrImage, []) => Ok(SlotKind::VideoOrImage),
        (Op::SlotText, []) => Ok(SlotKind::Text),
        _ => Err(malformed()),
    }
}

fn fill(graph: &FrozenDomainGraph, source: &Value) -> Result<FillMode, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::FillFitDuration, []) => Ok(FillMode::FitDuration),
        (Op::FillTakeHead, []) => Ok(FillMode::TakeHead),
        (Op::FillTakeCenter, []) => Ok(FillMode::TakeCenter),
        _ => Err(malformed()),
    }
}

fn duration(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
) -> Result<Option<veac_ir::RationalTime>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::SourceDurationAny, []) => Ok(None),
        (Op::SourceDurationAtLeast, [value]) => Ok(Some(time::coordinate(Some(value), timebase)?)),
        _ => Err(malformed()),
    }
}

fn text_policy(graph: &FrozenDomainGraph, source: &Value) -> Result<bool, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::TemplateTextLocked, []) => Ok(false),
        (Op::TemplateTextEditable, []) => Ok(true),
        _ => Err(malformed()),
    }
}

fn malformed() -> ExecutableLowerError {
    value::graph("an executable template contract has invalid typed topology")
}
