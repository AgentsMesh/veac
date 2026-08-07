use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{TextPath, TextPathAlignment};

use super::super::{animation, value};
use super::{invalid, ExecutableLowerError};

pub(super) fn choice(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<TextPath>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::TextPathNone, []) => Ok(None),
        (Op::TextPathPresent, [path]) => lower(graph, path).map(Some),
        _ => Err(invalid()),
    }
}

fn lower(graph: &FrozenDomainGraph, source: &Value) -> Result<TextPath, ExecutableLowerError> {
    let operands = value::description_operands(graph, source, Op::TextPath)?;
    let [points, offset, reverse, alignment] = operands else {
        return Err(invalid());
    };
    let values = value::list(Some(points))?;
    if !(2..=256).contains(&values.len()) {
        return Err(invalid());
    }
    let points = values
        .iter()
        .map(|point| animation::point_value(graph, Some(point)))
        .collect::<Result<Vec<_>, _>>()?;
    if points.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(invalid());
    }
    Ok(TextPath {
        points,
        start_offset: animation::length_value(graph, Some(offset))?,
        reverse: value::boolean(Some(reverse))?,
        alignment: align(graph, alignment)?,
    })
}

fn align(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<TextPathAlignment, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::TextPathAlignStart, []) => Ok(TextPathAlignment::Start),
        (Op::TextPathAlignCenter, []) => Ok(TextPathAlignment::Center),
        (Op::TextPathAlignEnd, []) => Ok(TextPathAlignment::End),
        _ => Err(invalid()),
    }
}
