use veac_plan::canonical::{ColorCurves, ToneCurve, ToneCurveInterpolation};

use super::{filter, number};
use crate::emitter::EmitContext;

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    mut input: String,
    value: &ColorCurves,
) -> String {
    for interpolation in ordered_interpolations(value) {
        if let Some(expression) = expression(value, interpolation) {
            input = filter(context, input, expression, "curvesv");
        }
    }
    input
}

fn ordered_interpolations(value: &ColorCurves) -> [ToneCurveInterpolation; 2] {
    match value.luma.as_ref().map(|curve| curve.interpolation) {
        Some(ToneCurveInterpolation::Monotonic) => [
            ToneCurveInterpolation::Monotonic,
            ToneCurveInterpolation::Natural,
        ],
        _ => [
            ToneCurveInterpolation::Natural,
            ToneCurveInterpolation::Monotonic,
        ],
    }
}

fn expression(value: &ColorCurves, interpolation: ToneCurveInterpolation) -> Option<String> {
    let curves = [&value.luma, &value.red, &value.green, &value.blue];
    let mut options = Vec::new();
    for (name, curve) in ["master", "red", "green", "blue"].into_iter().zip(curves) {
        if let Some(curve) = curve {
            if curve.interpolation == interpolation {
                options.push(format!("{name}='{}'", points(curve)));
            }
        }
    }
    if options.is_empty() {
        return None;
    }
    options.push(format!("interp={}", name(interpolation)));
    Some(format!("curves={}", options.join(":")))
}

fn points(value: &ToneCurve) -> String {
    value
        .points
        .iter()
        .map(|point| format!("{}/{}", number(point.input), number(point.output)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn name(value: ToneCurveInterpolation) -> &'static str {
    match value {
        ToneCurveInterpolation::Natural => "natural",
        ToneCurveInterpolation::Monotonic => "pchip",
    }
}
