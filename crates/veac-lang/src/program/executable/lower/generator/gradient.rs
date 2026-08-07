use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{Gradient, GradientStop};

use super::super::error::ExecutableLowerError;
use super::super::{animation, value};
use super::{malformed, vector};

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Gradient, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::GradientLinear, [start, end, stops]) => Ok(Gradient::Linear {
            start: vector(graph, start)?,
            end: vector(graph, end)?,
            stops: list(graph, stops)?,
        }),
        (Op::GradientRadial, [center, radius, stops]) => Ok(Gradient::Radial {
            center: vector(graph, center)?,
            radius: value::finite(Some(radius))?,
            stops: list(graph, stops)?,
        }),
        _ => Err(malformed()),
    }
}

fn list(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Vec<GradientStop>, ExecutableLowerError> {
    value::list(Some(source))?
        .iter()
        .map(|source| stop(graph, source))
        .collect()
}

fn stop(graph: &FrozenDomainGraph, source: &Value) -> Result<GradientStop, ExecutableLowerError> {
    let operands = value::description_operands(graph, source, Op::GradientStop)?;
    if operands.len() != 2 {
        return Err(malformed());
    }
    Ok(GradientStop {
        offset: animation::percent_value(graph, operands.first())?,
        color: value::color(operands.get(1))?,
    })
}
