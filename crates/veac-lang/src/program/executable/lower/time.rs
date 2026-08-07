use crate::program::expression::{ExactNumber, Value};
use crate::program::DomainOperationId;

use super::error::ExecutableLowerError;
use super::value;
use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;

pub(super) fn range(
    graph: &FrozenDomainGraph,
    value: &Value,
    timebase: u32,
) -> Result<veac_ir::TimeRange, ExecutableLowerError> {
    let operands = value::description_operands(graph, value, DomainOperationId::During)?;
    let start = value::exact(operands.first())?;
    let duration = value::exact(operands.get(1))?;
    if start.numerator() < 0 || duration.numerator() <= 0 {
        return Err(time_error());
    }
    veac_ir::TimeRange::new(ticks(start, timebase)?, ticks(duration, timebase)?)
        .map_err(|_| time_error())
}

fn ticks(value: ExactNumber, timebase: u32) -> Result<veac_ir::RationalTime, ExecutableLowerError> {
    let denominator = u32::try_from(value.denominator()).map_err(|_| time_error())?;
    if denominator == 0 || timebase % denominator != 0 {
        return Err(time_error());
    }
    let scale = i128::from(timebase / denominator);
    let ticks = value
        .numerator()
        .checked_mul(scale)
        .ok_or_else(time_error)?;
    let ticks = i64::try_from(ticks).map_err(|_| time_error())?;
    veac_ir::RationalTime::new(ticks, timebase).map_err(|_| time_error())
}

pub(super) fn coordinate(
    value: Option<&Value>,
    timebase: u32,
) -> Result<veac_ir::RationalTime, ExecutableLowerError> {
    ticks(value::exact(value)?, timebase)
}

pub(super) fn intrinsic(
    value: Option<&Value>,
) -> Result<veac_ir::RationalTime, ExecutableLowerError> {
    let value = value::exact(value)?;
    veac_ir::RationalTime::new(
        i64::try_from(value.numerator()).map_err(|_| time_error())?,
        u32::try_from(value.denominator()).map_err(|_| time_error())?,
    )
    .map_err(|_| time_error())
}

fn time_error() -> ExecutableLowerError {
    ExecutableLowerError::lower(
        "EXECUTABLE_LOWER_TIME",
        "an executable clip requires an exact non-negative start and positive safe duration",
    )
}
