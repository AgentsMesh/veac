mod animation;
mod decoration;
mod font;
mod layout;
mod path;
mod span;

use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{RationalTime, TextStyle};

use super::error::ExecutableLowerError;
use super::value;

pub(super) fn style(
    graph: &FrozenDomainGraph,
    source: &Value,
    content: &str,
    timebase: u32,
    duration: RationalTime,
    scope: &[&str],
) -> Result<TextStyle, ExecutableLowerError> {
    if content.is_empty()
        || content.len() > veac_ir::MAX_TEXT_BYTES
        || content.chars().count() > veac_ir::MAX_TEXT_SCALARS
    {
        return Err(invalid());
    }
    let operands = value::description_operands(graph, source, Op::TextStyle)?;
    let [metrics, text_layout, text_path, decorations, spans, text_animation] = operands else {
        return Err(invalid());
    };
    let metrics = font::metrics(graph, metrics)?;
    let layout = layout::lower(graph, text_layout)?;
    let path = path::choice(graph, text_path)?;
    let decoration = decoration::lower(graph, decorations)?;
    if path.is_some()
        && (layout.writing_mode != veac_ir::TextWritingMode::HorizontalTb
            || layout.wrap != veac_ir::TextWrap::None)
    {
        return Err(invalid());
    }
    Ok(TextStyle {
        font: metrics.font,
        fallback_fonts: metrics.fallback_fonts,
        font_weight: metrics.weight,
        font_style: metrics.style,
        size_pixels: metrics.size,
        color: metrics.fill,
        tracking_pixels: metrics.tracking,
        line_height: metrics.line_height,
        layout,
        path,
        background: decoration.background,
        outline: decoration.outline,
        shadow: decoration.shadow,
        spans: span::lower(graph, spans, content)?,
        animation: animation::choice(graph, text_animation, timebase, duration, scope)?,
    })
}

pub(super) fn invalid() -> ExecutableLowerError {
    ExecutableLowerError::lower(
        "EXECUTABLE_LOWER_TEXT",
        "a typed text source is invalid or cannot be represented in canonical IR",
    )
}
