use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{Shadow, TextBackground, TextOutline};

use super::super::{animation, value};
use super::{invalid, ExecutableLowerError};

pub(super) struct Decoration {
    pub(super) background: Option<TextBackground>,
    pub(super) outline: Option<TextOutline>,
    pub(super) shadow: Option<Shadow>,
}

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Decoration, ExecutableLowerError> {
    let operands = value::description_operands(graph, source, Op::TextDecoration)?;
    let [background, outline, shadow] = operands else {
        return Err(invalid());
    };
    Ok(Decoration {
        background: background_value(graph, background)?,
        outline: outline_value(graph, outline)?,
        shadow: shadow_value(graph, shadow)?,
    })
}

fn background_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<TextBackground>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::TextBackgroundNone, []) => Ok(None),
        (Op::TextBackgroundPresent, [fill, padding]) => {
            let padding_pixels = animation::length_value(graph, Some(padding))?.value;
            if !(0.0..=veac_ir::MAX_TEXT_PADDING_PIXELS).contains(&padding_pixels) {
                return Err(invalid());
            }
            Ok(Some(TextBackground {
                color: value::color(Some(fill))?,
                padding_pixels,
            }))
        }
        _ => Err(invalid()),
    }
}

fn outline_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<TextOutline>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::TextOutlineNone, []) => Ok(None),
        (Op::TextOutlinePresent, [color, width]) => {
            let width_pixels = animation::length_value(graph, Some(width))?.value;
            if !(0.0..=veac_ir::MAX_TEXT_OUTLINE_PIXELS).contains(&width_pixels) {
                return Err(invalid());
            }
            Ok(Some(TextOutline {
                color: value::color(Some(color))?,
                width_pixels,
            }))
        }
        _ => Err(invalid()),
    }
}

fn shadow_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<Shadow>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::ShadowNone, []) => Ok(None),
        (Op::ShadowPresent, [blur, opacity, offset, color]) => {
            let blur_pixels = animation::length_value(graph, Some(blur))?.value;
            if !veac_ir::shadow_blur_valid(blur_pixels) {
                return Err(invalid());
            }
            Ok(Some(Shadow {
                blur_pixels,
                opacity: animation::percent_value(graph, Some(opacity))?,
                offset: animation::vector_value(graph, Some(offset))?,
                color: value::color(Some(color))?,
            }))
        }
        _ => Err(invalid()),
    }
}
