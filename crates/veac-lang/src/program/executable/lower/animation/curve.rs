use std::collections::BTreeSet;

use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{Animatable, Keyframe, KeyframeId};

use super::super::{id, time, value};
use super::{interpolation, invalid, ExecutableLowerError};

#[derive(Clone, Copy)]
struct Operations {
    constant: Op,
    keyframes: Op,
    keyframe: Op,
}

macro_rules! curve {
    ($name:ident, $target:ty, $constant:ident, $keyframes:ident, $keyframe:ident, $decode:ident) => {
        pub(in crate::program::executable::lower) fn $name(
            graph: &FrozenDomainGraph,
            source: &Value,
            timebase: u32,
            scope: &[&str],
        ) -> Result<Animatable<$target>, ExecutableLowerError> {
            lower(
                graph,
                source,
                timebase,
                scope,
                Operations {
                    constant: Op::$constant,
                    keyframes: Op::$keyframes,
                    keyframe: Op::$keyframe,
                },
                super::$decode,
            )
        }
    };
}

curve!(
    scalar,
    f64,
    ScalarConstant,
    ScalarKeyframes,
    ScalarKeyframe,
    scalar_value
);
curve!(
    length,
    veac_ir::Length,
    LengthConstant,
    LengthKeyframes,
    LengthKeyframe,
    length_value
);
curve!(
    percent,
    f64,
    PercentConstant,
    PercentKeyframes,
    PercentKeyframe,
    percent_value
);
curve!(
    angle,
    f64,
    AngleConstant,
    AngleKeyframes,
    AngleKeyframe,
    angle_value
);
curve!(
    point,
    veac_ir::Point,
    PointConstant,
    PointKeyframes,
    PointKeyframe,
    point_value
);
curve!(
    vector,
    veac_ir::Vec2,
    VectorConstant,
    VectorKeyframes,
    VectorKeyframe,
    vector_value
);
curve!(
    rect,
    veac_ir::Rect,
    RectConstant,
    RectKeyframes,
    RectKeyframe,
    rect_value
);

fn lower<T>(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    scope: &[&str],
    operations: Operations,
    decode: fn(&FrozenDomainGraph, Option<&Value>) -> Result<T, ExecutableLowerError>,
) -> Result<Animatable<T>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (operation, [value]) if operation == operations.constant => {
            Ok(Animatable::constant(decode(graph, Some(value))?))
        }
        (operation, [values]) if operation == operations.keyframes => {
            let values = value::list(Some(values))?;
            if values.is_empty() {
                return Err(invalid());
            }
            let mut keys = BTreeSet::new();
            let mut previous = None;
            let mut keyframes = Vec::with_capacity(values.len());
            for source in values {
                let operands = value::description_operands(graph, source, operations.keyframe)?;
                let [key, at, source, easing] = operands else {
                    return Err(invalid());
                };
                let key = value::identifier(Some(key))?;
                let at = time::coordinate(Some(at), timebase)?;
                if at.value < 0
                    || !keys.insert(key)
                    || previous.is_some_and(|previous| at <= previous)
                {
                    return Err(invalid());
                }
                previous = Some(at);
                keyframes.push(Keyframe {
                    id: keyframe_id(scope, key),
                    time: at,
                    value: decode(graph, Some(source))?,
                    interpolation: interpolation::lower(graph, easing)?,
                });
            }
            Ok(Animatable::Keyframes { keyframes })
        }
        _ => Err(invalid()),
    }
}

fn keyframe_id(scope: &[&str], key: &str) -> KeyframeId {
    let mut path = Vec::with_capacity(scope.len() + 1);
    path.extend_from_slice(scope);
    path.push(key);
    id::keyframe(&path)
}
