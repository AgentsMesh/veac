use crate::program::expression::runtime::domain_graph::{FrozenDomainGraph, FrozenEntity};
use crate::program::{DomainOperationId as Op, DomainType};

use super::error::ExecutableLowerError;
use super::value;

mod track;

#[derive(Debug, Clone, Copy)]
pub(super) struct ProjectSettings {
    pub(super) timebase: u32,
}

pub(super) fn project(
    graph: &FrozenDomainGraph,
    root: FrozenEntity<'_>,
) -> Result<ProjectSettings, ExecutableLowerError> {
    let operands = value::entity_operands(root, DomainType::Project, Op::Project)?;
    let settings = value::description_operands(
        graph,
        operands.get(1).ok_or_else(malformed)?,
        Op::ProjectSettings,
    )?;
    let timebase = positive_u32(settings.first())?;
    Ok(ProjectSettings { timebase })
}

pub(super) fn sequence(
    graph: &FrozenDomainGraph,
    entity: FrozenEntity<'_>,
) -> Result<veac_ir::SequenceSettings, ExecutableLowerError> {
    let operands = value::entity_operands(entity, DomainType::Sequence, Op::Sequence)?;
    let settings = value::description_operands(
        graph,
        operands.get(2).ok_or_else(malformed)?,
        Op::SequenceSettings,
    )?;
    let canvas =
        value::description_operands(graph, settings.first().ok_or_else(malformed)?, Op::Canvas)?;
    let rate =
        value::description_operands(graph, settings.get(1).ok_or_else(malformed)?, Op::FrameRate)?;
    Ok(veac_ir::SequenceSettings {
        width: dimension(canvas.first())?,
        height: dimension(canvas.get(1))?,
        frame_rate: veac_ir::Rational::new(positive_i64(rate.first())?, positive_u32(rate.get(1))?)
            .map_err(|_| malformed())?,
        sample_rate: positive_u32(settings.get(2))?,
    })
}

pub(super) use track::layer;

fn positive_i64(
    value: Option<&crate::program::expression::Value>,
) -> Result<i64, ExecutableLowerError> {
    let value = value::integer(value)?;
    (value > 0).then_some(value).ok_or_else(malformed)
}

fn positive_u32(
    value: Option<&crate::program::expression::Value>,
) -> Result<u32, ExecutableLowerError> {
    u32::try_from(positive_i64(value)?).map_err(|_| malformed())
}

fn dimension(
    value: Option<&crate::program::expression::Value>,
) -> Result<u32, ExecutableLowerError> {
    let exact = value::exact(value)?;
    if exact.numerator() <= 0 || exact.denominator() != 1 {
        return Err(malformed());
    }
    u32::try_from(exact.numerator()).map_err(|_| malformed())
}

fn malformed() -> ExecutableLowerError {
    value::graph("the executable graph has invalid project or sequence settings")
}
