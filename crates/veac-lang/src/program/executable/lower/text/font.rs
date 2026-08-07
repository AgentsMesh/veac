use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{FontRef, FontStyle, FontWeight};

use super::super::{animation, id, value};
use super::{invalid, ExecutableLowerError};

pub(super) struct Metrics {
    pub(super) font: FontRef,
    pub(super) fallback_fonts: Vec<FontRef>,
    pub(super) weight: FontWeight,
    pub(super) style: FontStyle,
    pub(super) size: f64,
    pub(super) tracking: f64,
    pub(super) line_height: f64,
    pub(super) fill: veac_ir::Color,
}

pub(super) struct RunStyle {
    pub(super) font: FontRef,
    pub(super) weight: FontWeight,
    pub(super) style: FontStyle,
    pub(super) size: f64,
    pub(super) fill: veac_ir::Color,
}

pub(super) fn metrics(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Metrics, ExecutableLowerError> {
    let operands = value::description_operands(graph, source, Op::TextMetrics)?;
    let [fonts, weight_source, style_source, size, tracking, line_height, fill] = operands else {
        return Err(invalid());
    };
    let (font, fallback_fonts) = stack(graph, fonts)?;
    let size = length(graph, size)?;
    let tracking = length(graph, tracking)?;
    let line_height = scalar(graph, line_height)?;
    if size <= 0.0
        || size > veac_ir::MAX_TEXT_SIZE_PIXELS
        || tracking.abs() > veac_ir::MAX_TEXT_TRACKING_PIXELS
        || line_height <= 0.0
        || line_height > veac_ir::MAX_TEXT_LINE_HEIGHT
    {
        return Err(invalid());
    }
    Ok(Metrics {
        font,
        fallback_fonts,
        weight: weight(graph, weight_source)?,
        style: style(graph, style_source)?,
        size,
        tracking,
        line_height,
        fill: value::color(Some(fill))?,
    })
}

pub(super) fn run_style(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<RunStyle, ExecutableLowerError> {
    let operands = value::description_operands(graph, source, Op::TextRunStyle)?;
    let [font_source, weight_source, style_source, size, fill] = operands else {
        return Err(invalid());
    };
    let size = length(graph, size)?;
    if size <= 0.0 || size > veac_ir::MAX_TEXT_SIZE_PIXELS {
        return Err(invalid());
    }
    Ok(RunStyle {
        font: font(graph, font_source)?,
        weight: weight(graph, weight_source)?,
        style: style(graph, style_source)?,
        size,
        fill: value::color(Some(fill))?,
    })
}

fn stack(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<(FontRef, Vec<FontRef>), ExecutableLowerError> {
    let operands = value::description_operands(graph, source, Op::FontStack)?;
    let [primary, fallbacks] = operands else {
        return Err(invalid());
    };
    let primary = font(graph, primary)?;
    let values = value::list(Some(fallbacks))?;
    if values.len() > veac_ir::MAX_FALLBACK_FONTS {
        return Err(invalid());
    }
    let mut lowered = Vec::with_capacity(values.len());
    for source in values {
        let fallback = font(graph, source)?;
        if fallback == primary || lowered.contains(&fallback) {
            return Err(invalid());
        }
        lowered.push(fallback);
    }
    Ok((primary, lowered))
}

pub(super) fn font(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<FontRef, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::FontFamily, [name]) => {
            let family = value::text(Some(name))?;
            if family.trim().is_empty() {
                return Err(invalid());
            }
            Ok(FontRef::Family {
                family: family.to_owned(),
            })
        }
        (Op::FontResourceRef, [Value::Domain(resource)])
            if graph.operation(resource) == Some(Op::FontResource) =>
        {
            let path = graph.logical_key(resource).ok_or_else(invalid)?;
            Ok(FontRef::Material {
                material_id: id::material(&path),
            })
        }
        _ => Err(invalid()),
    }
}

fn weight(graph: &FrozenDomainGraph, source: &Value) -> Result<FontWeight, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::WeightThin, []) => Ok(FontWeight::Thin),
        (Op::WeightExtraLight, []) => Ok(FontWeight::ExtraLight),
        (Op::WeightLight, []) => Ok(FontWeight::Light),
        (Op::WeightNormal, []) => Ok(FontWeight::Normal),
        (Op::WeightMedium, []) => Ok(FontWeight::Medium),
        (Op::WeightSemiBold, []) => Ok(FontWeight::SemiBold),
        (Op::WeightBold, []) => Ok(FontWeight::Bold),
        (Op::WeightExtraBold, []) => Ok(FontWeight::ExtraBold),
        (Op::WeightBlack, []) => Ok(FontWeight::Black),
        _ => Err(invalid()),
    }
}

fn style(graph: &FrozenDomainGraph, source: &Value) -> Result<FontStyle, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::FontStyleNormal, []) => Ok(FontStyle::Normal),
        (Op::FontStyleItalic, []) => Ok(FontStyle::Italic),
        (Op::FontStyleOblique, []) => Ok(FontStyle::Oblique),
        _ => Err(invalid()),
    }
}

fn length(graph: &FrozenDomainGraph, source: &Value) -> Result<f64, ExecutableLowerError> {
    Ok(animation::length_value(graph, Some(source))?.value)
}

fn scalar(graph: &FrozenDomainGraph, source: &Value) -> Result<f64, ExecutableLowerError> {
    animation::scalar_value(graph, Some(source))
}
