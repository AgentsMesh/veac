use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::Generator;

use super::error::ExecutableLowerError;
use super::value;

mod gradient;
mod shape;

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Generator, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::GeneratorSolid, [color]) => Ok(Generator::Solid {
            color: value::color(Some(color))?,
        }),
        (Op::GeneratorGradient, [gradient]) => Ok(Generator::Gradient {
            gradient: gradient::lower(graph, gradient)?,
        }),
        (Op::VectorShape, [geometry, fill, stroke]) => Ok(Generator::Shape {
            shape: shape::lower(graph, geometry, fill, stroke)?,
        }),
        (Op::GeneratorTransparent, []) => Ok(Generator::Transparent),
        (Op::GeneratorSilence, []) => Ok(Generator::Silence),
        _ => Err(malformed()),
    }
}

pub(super) fn vector(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<veac_ir::Vec2, ExecutableLowerError> {
    let operands = value::description_operands(graph, source, Op::Vector)?;
    if operands.len() != 2 {
        return Err(malformed());
    }
    Ok(veac_ir::Vec2 {
        x: value::finite(operands.first())?,
        y: value::finite(operands.get(1))?,
    })
}

pub(super) fn rect(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<veac_ir::Rect, ExecutableLowerError> {
    let operands = value::description_operands(graph, source, Op::Rect)?;
    if operands.len() != 4 {
        return Err(malformed());
    }
    Ok(veac_ir::Rect {
        x: value::finite(operands.first())?,
        y: value::finite(operands.get(1))?,
        width: value::finite(operands.get(2))?,
        height: value::finite(operands.get(3))?,
    })
}

fn malformed() -> ExecutableLowerError {
    value::graph("an executable generator has invalid typed geometry or paint")
}
