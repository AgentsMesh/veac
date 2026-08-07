use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{ColorCurves, CurvePoint, ToneCurve, ToneCurveInterpolation};

use super::{malformed, ExecutableLowerError};
use crate::program::executable::lower::{animation, value};

pub(super) fn curves(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<ColorCurves, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::ColorCurves)?;
    if values.len() != 4 {
        return Err(malformed());
    }
    Ok(ColorCurves {
        luma: channel(graph, &values[0])?,
        red: channel(graph, &values[1])?,
        green: channel(graph, &values[2])?,
        blue: channel(graph, &values[3])?,
    })
}

fn channel(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<ToneCurve>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::ToneChannelNone, []) => Ok(None),
        (Op::ToneChannelPresent, [value]) => Ok(Some(curve(graph, value)?)),
        _ => Err(malformed()),
    }
}

fn curve(graph: &FrozenDomainGraph, source: &Value) -> Result<ToneCurve, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::ToneCurve)?;
    if values.len() != 2 {
        return Err(malformed());
    }
    Ok(ToneCurve {
        points: value::list(values.first())?
            .iter()
            .map(|point| point_value(graph, point))
            .collect::<Result<_, _>>()?,
        interpolation: interpolation(graph, &values[1])?,
    })
}

fn point_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<CurvePoint, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::CurvePoint)?;
    if values.len() != 2 {
        return Err(malformed());
    }
    Ok(CurvePoint {
        input: animation::percent_value(graph, values.first())?,
        output: animation::percent_value(graph, values.get(1))?,
    })
}

fn interpolation(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<ToneCurveInterpolation, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::ToneNatural, []) => Ok(ToneCurveInterpolation::Natural),
        (Op::ToneMonotonic, []) => Ok(ToneCurveInterpolation::Monotonic),
        _ => Err(malformed()),
    }
}
