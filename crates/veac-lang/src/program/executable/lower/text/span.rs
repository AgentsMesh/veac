use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::TextSpan;

use super::super::value;
use super::{font, invalid, ExecutableLowerError};

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
    content: &str,
) -> Result<Vec<TextSpan>, ExecutableLowerError> {
    let values = value::list(Some(source))?;
    if values.len() > veac_ir::MAX_TEXT_SPANS {
        return Err(invalid());
    }
    let content_length = content.chars().count() as i64;
    let mut previous_end = 0_i64;
    let mut spans = Vec::with_capacity(values.len());
    for source in values {
        let operands = value::description_operands(graph, source, Op::TextSpan)?;
        let [start, end, style] = operands else {
            return Err(invalid());
        };
        let start = value::integer(Some(start))?;
        let end = value::integer(Some(end))?;
        if start < previous_end || start < 0 || start >= end || end > content_length {
            return Err(invalid());
        }
        previous_end = end;
        let style = font::run_style(graph, style)?;
        spans.push(TextSpan {
            start: start as u32,
            end: end as u32,
            font: Some(style.font),
            font_weight: Some(style.weight),
            font_style: Some(style.style),
            size_pixels: Some(style.size),
            color: Some(style.fill),
        });
    }
    Ok(spans)
}
