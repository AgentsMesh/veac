use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{
    HorizontalTextAlignment, TextLayout, TextOrientation, TextOverflow, TextWrap, TextWritingMode,
    VerticalTextAlignment,
};

use super::super::{animation, value};
use super::{invalid, ExecutableLowerError};

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<TextLayout, ExecutableLowerError> {
    let operands = value::description_operands(graph, source, Op::TextLayout)?;
    let [text_box, wrap, overflow, horizontal, vertical, writing, orientation] = operands else {
        return Err(invalid());
    };
    let (box_width_pixels, box_height_pixels) = text_box_value(graph, text_box)?;
    let wrap = wrap_value(graph, wrap)?;
    let overflow = overflow_value(graph, overflow)?;
    let writing_mode = writing_value(graph, writing)?;
    let inline_bounded = match writing_mode {
        TextWritingMode::HorizontalTb => box_width_pixels.is_some(),
        TextWritingMode::VerticalRl | TextWritingMode::VerticalLr => box_height_pixels.is_some(),
    };
    if (wrap != TextWrap::None || overflow != TextOverflow::Visible) && !inline_bounded {
        return Err(invalid());
    }
    Ok(TextLayout {
        box_width_pixels,
        box_height_pixels,
        wrap,
        overflow,
        horizontal_alignment: horizontal_value(graph, horizontal)?,
        vertical_alignment: vertical_value(graph, vertical)?,
        writing_mode,
        orientation: orientation_value(graph, orientation)?,
    })
}

fn text_box_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<(Option<f64>, Option<f64>), ExecutableLowerError> {
    let value = match value::description(graph, source)? {
        (Op::TextBoxAuto, []) => (None, None),
        (Op::TextBoxWidth, [width]) => (Some(length(graph, width)?), None),
        (Op::TextBoxHeight, [height]) => (None, Some(length(graph, height)?)),
        (Op::TextBoxFixed, [width, height]) => {
            (Some(length(graph, width)?), Some(length(graph, height)?))
        }
        _ => return Err(invalid()),
    };
    veac_ir::text_box_valid(value.0, value.1)
        .then_some(value)
        .ok_or_else(invalid)
}

fn wrap_value(graph: &FrozenDomainGraph, source: &Value) -> Result<TextWrap, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::TextWrapNone, []) => Ok(TextWrap::None),
        (Op::TextWrapWord, []) => Ok(TextWrap::Word),
        (Op::TextWrapCharacter, []) => Ok(TextWrap::Character),
        _ => Err(invalid()),
    }
}

fn overflow_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<TextOverflow, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::TextOverflowVisible, []) => Ok(TextOverflow::Visible),
        (Op::TextOverflowClip, []) => Ok(TextOverflow::Clip),
        (Op::TextOverflowEllipsis, []) => Ok(TextOverflow::Ellipsis),
        _ => Err(invalid()),
    }
}

fn horizontal_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<HorizontalTextAlignment, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::TextAlignLeft, []) => Ok(HorizontalTextAlignment::Left),
        (Op::TextAlignCenter, []) => Ok(HorizontalTextAlignment::Center),
        (Op::TextAlignRight, []) => Ok(HorizontalTextAlignment::Right),
        _ => Err(invalid()),
    }
}

fn vertical_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<VerticalTextAlignment, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::TextAlignTop, []) => Ok(VerticalTextAlignment::Top),
        (Op::TextAlignMiddle, []) => Ok(VerticalTextAlignment::Middle),
        (Op::TextAlignBottom, []) => Ok(VerticalTextAlignment::Bottom),
        _ => Err(invalid()),
    }
}

fn writing_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<TextWritingMode, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::WritingHorizontalTb, []) => Ok(TextWritingMode::HorizontalTb),
        (Op::WritingVerticalRl, []) => Ok(TextWritingMode::VerticalRl),
        (Op::WritingVerticalLr, []) => Ok(TextWritingMode::VerticalLr),
        _ => Err(invalid()),
    }
}

fn orientation_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<TextOrientation, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::OrientationMixed, []) => Ok(TextOrientation::Mixed),
        (Op::OrientationUpright, []) => Ok(TextOrientation::Upright),
        (Op::OrientationSideways, []) => Ok(TextOrientation::Sideways),
        _ => Err(invalid()),
    }
}

fn length(graph: &FrozenDomainGraph, source: &Value) -> Result<f64, ExecutableLowerError> {
    Ok(animation::length_value(graph, Some(source))?.value)
}
