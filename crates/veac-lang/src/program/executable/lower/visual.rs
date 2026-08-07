use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::VisualProperties;

use super::error::ExecutableLowerError;
use super::{color, value};

mod layout;
mod mask;
mod surface;

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    scope: &[&str],
) -> Result<VisualProperties, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::VisualStyle)?;
    if values.len() != 4 {
        return Err(malformed());
    }
    let layout = layout::lower(graph, &values[0], timebase, scope)?;
    let surface = surface::lower(graph, &values[1], timebase, scope)?;
    Ok(VisualProperties {
        placement: layout.placement,
        frame: layout.frame,
        transform: layout.transform,
        opacity: surface.opacity,
        compositing: surface.compositing,
        masks: value::list(values.get(2))?
            .iter()
            .enumerate()
            .map(|(index, source)| mask::lower(graph, source, timebase, scope, index))
            .collect::<Result<_, _>>()?,
        card: surface.card,
        color_pipeline: color::choice(graph, &values[3])?,
    })
}

pub(super) fn channel<'a>(scope: &[&'a str], name: &'a str) -> Vec<&'a str> {
    let mut result = Vec::with_capacity(scope.len() + 1);
    result.extend_from_slice(scope);
    result.push(name);
    result
}

pub(super) fn blend(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<veac_ir::BlendMode, ExecutableLowerError> {
    surface::blend(graph, source)
}

pub(super) fn masks(
    graph: &FrozenDomainGraph,
    sources: &[Value],
    timebase: u32,
    scope: &[&str],
) -> Result<Vec<veac_ir::Mask>, ExecutableLowerError> {
    sources
        .iter()
        .enumerate()
        .map(|(index, source)| mask::lower(graph, source, timebase, scope, index))
        .collect()
}

fn malformed() -> ExecutableLowerError {
    value::graph("an executable visual style has invalid typed topology")
}
