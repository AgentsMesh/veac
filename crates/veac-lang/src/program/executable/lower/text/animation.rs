use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{
    Animatable, RationalTime, TextAnimation, TextGranularity, TextHighlightAnimation,
    TextUnitTransform, Vec2,
};

use super::super::{animation, time, value};
use super::{invalid, ExecutableLowerError};

pub(super) fn choice(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    duration: RationalTime,
    scope: &[&str],
) -> Result<Option<TextAnimation>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::TextAnimationNone, []) => Ok(None),
        (Op::TextAnimationPresent, [animation]) => {
            lower(graph, animation, timebase, duration, scope).map(Some)
        }
        _ => Err(invalid()),
    }
}

fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    duration: RationalTime,
    scope: &[&str],
) -> Result<TextAnimation, ExecutableLowerError> {
    let operands = value::description_operands(graph, source, Op::TextAnimation)?;
    let [granularity, transform, reveal, highlight, opacity, stagger] = operands else {
        return Err(invalid());
    };
    let reveal = animation::percent(graph, reveal, timebase, &channel(scope, "text.reveal"))?;
    let opacity = animation::percent(graph, opacity, timebase, &channel(scope, "text.opacity"))?;
    let transform = transform_value(graph, transform, timebase, scope)?;
    let highlight = highlight_value(graph, highlight, timebase, scope)?;
    let stagger = time::coordinate(Some(stagger), timebase)?;
    if stagger.value < 0
        || !curve_valid(&reveal, duration, |value| (0.0..=1.0).contains(value))
        || !curve_valid(&opacity, duration, |value| (0.0..=1.0).contains(value))
        || !point_duration_valid(&transform.position_offset, duration)
        || !curve_valid(&transform.rotation_degrees, duration, |value| {
            value.is_finite()
        })
        || !vector_curve_valid(&transform.scale, duration)
        || highlight.as_ref().is_some_and(|highlight| {
            !curve_valid(&highlight.progress, duration, |value| {
                (0.0..=1.0).contains(value)
            })
        })
    {
        return Err(invalid());
    }
    Ok(TextAnimation {
        granularity: granularity_value(graph, granularity)?,
        transform,
        reveal,
        highlight,
        opacity,
        stagger,
    })
}

fn transform_value(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    scope: &[&str],
) -> Result<TextUnitTransform, ExecutableLowerError> {
    let operands = value::description_operands(graph, source, Op::TextUnitTransform)?;
    let [position, scale, rotation] = operands else {
        return Err(invalid());
    };
    Ok(TextUnitTransform {
        position_offset: animation::point(
            graph,
            position,
            timebase,
            &channel(scope, "text.position"),
        )?,
        scale: animation::vector(graph, scale, timebase, &channel(scope, "text.scale"))?,
        rotation_degrees: animation::angle(
            graph,
            rotation,
            timebase,
            &channel(scope, "text.rotation"),
        )?,
    })
}

fn highlight_value(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    scope: &[&str],
) -> Result<Option<TextHighlightAnimation>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::TextHighlightNone, []) => Ok(None),
        (Op::TextHighlightPresent, [fill, progress]) => Ok(Some(TextHighlightAnimation {
            fill: value::color(Some(fill))?,
            progress: animation::percent(
                graph,
                progress,
                timebase,
                &channel(scope, "text.highlight"),
            )?,
        })),
        _ => Err(invalid()),
    }
}

fn granularity_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<TextGranularity, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::TextWhole, []) => Ok(TextGranularity::Whole),
        (Op::TextLine, []) => Ok(TextGranularity::Line),
        (Op::TextWord, []) => Ok(TextGranularity::Word),
        (Op::TextGrapheme, []) => Ok(TextGranularity::Grapheme),
        _ => Err(invalid()),
    }
}

fn channel<'a>(scope: &[&'a str], channel: &'a str) -> Vec<&'a str> {
    scope
        .iter()
        .copied()
        .chain(std::iter::once(channel))
        .collect()
}

fn curve_valid(
    curve: &Animatable<f64>,
    duration: RationalTime,
    valid: impl Fn(&f64) -> bool,
) -> bool {
    match curve {
        Animatable::Constant { value } => valid(value),
        Animatable::Binding { .. } => false,
        Animatable::Keyframes { keyframes } => {
            keyframes
                .iter()
                .all(|key| key.time <= duration && valid(&key.value))
                && keyframes.windows(2).all(|pair| {
                    pair[0]
                        .interpolation
                        .intermediate_extrema()
                        .is_none_or(|amounts| {
                            amounts.into_iter().all(|amount| {
                                valid(&(pair[0].value + (pair[1].value - pair[0].value) * amount))
                            })
                        })
                })
        }
    }
}

fn vector_curve_valid(curve: &Animatable<Vec2>, duration: RationalTime) -> bool {
    match curve {
        Animatable::Constant { value } => veac_ir::visual_scale_valid(*value),
        Animatable::Binding { .. } => false,
        Animatable::Keyframes { keyframes } => {
            keyframes
                .iter()
                .all(|key| key.time <= duration && veac_ir::visual_scale_valid(key.value))
                && keyframes.windows(2).all(|pair| {
                    pair[0]
                        .interpolation
                        .intermediate_extrema()
                        .is_none_or(|amounts| {
                            amounts.into_iter().all(|amount| {
                                veac_ir::visual_scale_valid(Vec2 {
                                    x: pair[0].value.x
                                        + (pair[1].value.x - pair[0].value.x) * amount,
                                    y: pair[0].value.y
                                        + (pair[1].value.y - pair[0].value.y) * amount,
                                })
                            })
                        })
                })
        }
    }
}

fn point_duration_valid(curve: &Animatable<veac_ir::Point>, duration: RationalTime) -> bool {
    match curve {
        Animatable::Constant { .. } => true,
        Animatable::Binding { .. } => false,
        Animatable::Keyframes { keyframes } => keyframes.iter().all(|key| key.time <= duration),
    }
}
