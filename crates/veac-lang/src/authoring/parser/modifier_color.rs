use crate::authoring::{
    BasicColorDecl, ColorModifierDecl, ColorStageDecl, HslColorDecl, Identifier, LutColorDecl,
    SemanticBlock, SemanticEntry,
};

use super::{color_space, color_stage, semantic, Parser};

pub(super) fn parse(
    parser: &mut Parser,
    id: Identifier,
    mut body: SemanticBlock,
    span: crate::authoring::Span,
) -> Option<ColorModifierDecl> {
    let input_entry = semantic::required(parser, &mut body, "input-space", "color modifier")?;
    let input = color_space::parse(parser, &input_entry)?;
    let working_entry = semantic::required(parser, &mut body, "working-space", "color modifier")?;
    let working = color_space::parse(parser, &working_entry)?;
    let output_entry = semantic::required(parser, &mut body, "output-space", "color modifier")?;
    let output = color_space::parse(parser, &output_entry)?;
    let stages = body
        .entries
        .drain(..)
        .filter_map(|entry| stage(parser, &entry))
        .collect();
    Some(ColorModifierDecl {
        id,
        input,
        working,
        output,
        stages,
        span,
    })
}

fn stage(parser: &mut Parser, entry: &SemanticEntry) -> Option<ColorStageDecl> {
    match entry.name.value.as_str() {
        "basic" => basic(parser, entry).map(ColorStageDecl::Basic),
        "matrix" => color_stage::matrix(parser, entry).map(ColorStageDecl::Matrix),
        "hsl" => hsl(parser, entry).map(ColorStageDecl::Hsl),
        "curves" => color_stage::curves(parser, entry).map(ColorStageDecl::Curves),
        "wheels" => color_stage::wheels(parser, entry).map(ColorStageDecl::Wheels),
        "lut" => lut(parser, entry).map(ColorStageDecl::Lut),
        _ => {
            parser.error(
                "AUTHORING_UNKNOWN_COLOR_STAGE",
                format!("unknown color stage '{}'", entry.name.value),
                entry.span,
            );
            None
        }
    }
}

fn basic(parser: &mut Parser, entry: &SemanticEntry) -> Option<BasicColorDecl> {
    let mut block = semantic::nested(parser, entry, "basic color stage")?;
    let result = BasicColorDecl {
        exposure: number(parser, &mut block, "exposure", "basic color")?,
        highlights: number(parser, &mut block, "highlights", "basic color")?,
        shadows: number(parser, &mut block, "shadows", "basic color")?,
        temperature: number(parser, &mut block, "temperature", "basic color")?,
        tint: number(parser, &mut block, "tint", "basic color")?,
        fade: number(parser, &mut block, "fade", "basic color")?,
        span: entry.span,
    };
    semantic::finish(parser, block, "basic color stage");
    Some(result)
}

fn hsl(parser: &mut Parser, entry: &SemanticEntry) -> Option<HslColorDecl> {
    let mut block = semantic::nested(parser, entry, "HSL color stage")?;
    let range_entry = semantic::required(parser, &mut block, "range", "HSL")?;
    let result = HslColorDecl {
        range: semantic::word(parser, &range_entry, "HSL range")?,
        hue: number(parser, &mut block, "hue", "HSL")?,
        saturation: number(parser, &mut block, "saturation", "HSL")?,
        lightness: number(parser, &mut block, "lightness", "HSL")?,
        span: entry.span,
    };
    semantic::finish(parser, block, "HSL color stage");
    Some(result)
}

fn lut(parser: &mut Parser, entry: &SemanticEntry) -> Option<LutColorDecl> {
    let mut block = semantic::nested(parser, entry, "LUT color stage")?;
    let resource_entry = semantic::required(parser, &mut block, "resource", "LUT stage")?;
    let resource = semantic::word(parser, &resource_entry, "LUT resource")?;
    let interpolation_entry = semantic::required(parser, &mut block, "interpolation", "LUT stage")?;
    let interpolation = semantic::word(parser, &interpolation_entry, "LUT interpolation")?;
    semantic::finish(parser, block, "LUT color stage");
    Some(LutColorDecl {
        resource,
        interpolation,
        span: entry.span,
    })
}

fn number(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
    context: &'static str,
) -> Option<crate::authoring::NumberLiteral> {
    let entry = semantic::required(parser, block, name, context)?;
    semantic::number(parser, &entry, name)
}
