use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{
    BasicColorAdjustment, ColorWheel, HslAdjustment, HueRange, LiftGammaGain, LutApplication,
    LutInterpolation, RgbMatrixAdjustment,
};

use super::{malformed, ExecutableLowerError};
use crate::program::executable::lower::{id, value};

pub(super) fn basic(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<BasicColorAdjustment, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::BasicColor)?;
    if values.len() != 6 {
        return Err(malformed());
    }
    Ok(BasicColorAdjustment {
        exposure_stops: value::finite(values.first())?,
        temperature_kelvin: value::finite(values.get(1))?,
        tint: value::finite(values.get(2))?,
        highlights: value::finite(values.get(3))?,
        shadows: value::finite(values.get(4))?,
        fade: value::finite(values.get(5))?,
    })
}

pub(super) fn matrix(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<RgbMatrixAdjustment, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::RgbMatrix)?;
    if values.len() != 2 {
        return Err(malformed());
    }
    Ok(RgbMatrixAdjustment {
        matrix: finite_array::<9>(values.first())?,
        offset: finite_array::<3>(values.get(1))?,
    })
}

pub(super) fn hsl(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<HslAdjustment, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::HslAdjustment)?;
    if values.len() != 4 {
        return Err(malformed());
    }
    Ok(HslAdjustment {
        range: hue(graph, &values[0])?,
        hue_degrees: value::finite(values.get(1))?,
        saturation: value::finite(values.get(2))?,
        lightness: value::finite(values.get(3))?,
    })
}

pub(super) fn wheels(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<LiftGammaGain, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::LiftGammaGain)?;
    if values.len() != 3 {
        return Err(malformed());
    }
    Ok(LiftGammaGain {
        lift: wheel(graph, &values[0])?,
        gamma: wheel(graph, &values[1])?,
        gain: wheel(graph, &values[2])?,
    })
}

pub(super) fn lut(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<LutApplication, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::LutApplication)?;
    if values.len() != 2 {
        return Err(malformed());
    }
    let Value::Domain(resource) = &values[0] else {
        return Err(malformed());
    };
    let path = graph.logical_key(resource).ok_or_else(malformed)?;
    Ok(LutApplication {
        material_id: id::material(&path),
        interpolation: interpolation(graph, &values[1])?,
    })
}

fn hue(graph: &FrozenDomainGraph, source: &Value) -> Result<HueRange, ExecutableLowerError> {
    Ok(match empty(graph, source)? {
        Op::HueRed => HueRange::Red,
        Op::HueYellow => HueRange::Yellow,
        Op::HueGreen => HueRange::Green,
        Op::HueCyan => HueRange::Cyan,
        Op::HueBlue => HueRange::Blue,
        Op::HueMagenta => HueRange::Magenta,
        _ => return Err(malformed()),
    })
}

fn wheel(graph: &FrozenDomainGraph, source: &Value) -> Result<ColorWheel, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::ColorWheel)?;
    if values.len() != 3 {
        return Err(malformed());
    }
    Ok(ColorWheel {
        red: value::finite(values.first())?,
        green: value::finite(values.get(1))?,
        blue: value::finite(values.get(2))?,
    })
}

fn interpolation(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<LutInterpolation, ExecutableLowerError> {
    Ok(match empty(graph, source)? {
        Op::LutNearest => LutInterpolation::Nearest,
        Op::LutLinear => LutInterpolation::Linear,
        Op::LutCosine => LutInterpolation::Cosine,
        Op::LutCubic => LutInterpolation::Cubic,
        Op::LutSpline => LutInterpolation::Spline,
        Op::LutTrilinear => LutInterpolation::Trilinear,
        Op::LutTetrahedral => LutInterpolation::Tetrahedral,
        Op::LutPyramid => LutInterpolation::Pyramid,
        Op::LutPrism => LutInterpolation::Prism,
        _ => return Err(malformed()),
    })
}

fn finite_array<const N: usize>(source: Option<&Value>) -> Result<[f64; N], ExecutableLowerError> {
    let values = value::list(source)?;
    if values.len() != N {
        return Err(malformed());
    }
    let values = values
        .iter()
        .map(|value| value::finite(Some(value)))
        .collect::<Result<Vec<_>, _>>()?;
    values.try_into().map_err(|_| malformed())
}

fn empty(graph: &FrozenDomainGraph, source: &Value) -> Result<Op, ExecutableLowerError> {
    let (operation, operands) = value::description(graph, source)?;
    operands
        .is_empty()
        .then_some(operation)
        .ok_or_else(malformed)
}
