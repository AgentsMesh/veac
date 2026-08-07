use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{ColorPipeline, ColorStage};

use super::error::ExecutableLowerError;
use super::value;

mod adjustment;
mod curve;
mod space;

pub(super) use space::lower as color_space;

pub(super) fn pipeline(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<ColorPipeline, ExecutableLowerError> {
    let operands = value::description_operands(graph, source, Op::ColorPipeline)?;
    if operands.len() != 4 {
        return Err(malformed());
    }
    Ok(ColorPipeline {
        input: space::lower(graph, &operands[0])?,
        working: space::lower(graph, &operands[1])?,
        output: space::lower(graph, &operands[2])?,
        stages: value::list(operands.get(3))?
            .iter()
            .map(|stage| stage_value(graph, stage))
            .collect::<Result<_, _>>()?,
    })
}

fn stage_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<ColorStage, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::ColorStageBasic, [value]) => Ok(ColorStage::Basic {
            adjustment: adjustment::basic(graph, value)?,
        }),
        (Op::ColorStageMatrix, [value]) => Ok(ColorStage::Matrix {
            adjustment: adjustment::matrix(graph, value)?,
        }),
        (Op::ColorStageHsl, [value]) => Ok(ColorStage::Hsl {
            adjustment: adjustment::hsl(graph, value)?,
        }),
        (Op::ColorStageCurves, [value]) => Ok(ColorStage::Curves {
            curves: curve::curves(graph, value)?,
        }),
        (Op::ColorStageWheels, [value]) => Ok(ColorStage::Wheels {
            wheels: adjustment::wheels(graph, value)?,
        }),
        (Op::ColorStageLut, [value]) => Ok(ColorStage::Lut {
            application: adjustment::lut(graph, value)?,
        }),
        _ => Err(malformed()),
    }
}

pub(super) fn choice(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<ColorPipeline>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::ColorPipelineNone, []) => Ok(None),
        (Op::ColorPipelinePresent, [value]) => Ok(Some(pipeline(graph, value)?)),
        _ => Err(malformed()),
    }
}

fn malformed() -> ExecutableLowerError {
    value::graph("an executable color pipeline has invalid typed topology")
}
