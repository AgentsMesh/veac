use crate::authoring::{
    SemanticEntry, TextAnimationDecl, TextAnimationTransformDecl, TextGranularityDecl,
    TextHighlightDecl,
};

use super::{parameter, semantic, text_value, Parser};

pub(super) fn parse(parser: &mut Parser, entry: &SemanticEntry) -> Option<TextAnimationDecl> {
    let mut block = semantic::nested(parser, entry, "text animation")?;
    let unit_entry = semantic::required(parser, &mut block, "unit", "text animation")?;
    let unit =
        text_value::enum_value(parser, &unit_entry, "animation unit", |value| match value {
            "whole" => Some(TextGranularityDecl::Whole),
            "line" => Some(TextGranularityDecl::Line),
            "word" => Some(TextGranularityDecl::Word),
            "grapheme" => Some(TextGranularityDecl::Grapheme),
            _ => None,
        })?;
    let stagger_entry = semantic::required(parser, &mut block, "stagger", "text animation")?;
    let stagger = semantic::number(parser, &stagger_entry, "text stagger")?;
    let reveal = scalar(parser, &mut block, "reveal")?;
    let highlight = match semantic::take(parser, &mut block, "highlight") {
        Some(entry) => Some(highlight(parser, &entry)?),
        None => None,
    };
    let opacity = scalar(parser, &mut block, "opacity")?;
    let transform = match semantic::take(parser, &mut block, "transform") {
        Some(entry) => Some(transform(parser, &entry)?),
        None => None,
    };
    semantic::finish(parser, block, "text animation");
    Some(TextAnimationDecl {
        unit,
        stagger,
        reveal,
        highlight,
        opacity,
        transform,
        span: entry.span,
    })
}

fn highlight(parser: &mut Parser, entry: &SemanticEntry) -> Option<TextHighlightDecl> {
    let mut block = semantic::nested(parser, entry, "text highlight")?;
    let fill_entry = semantic::required(parser, &mut block, "fill", "text highlight")?;
    let fill = text_value::color(parser, &fill_entry)?;
    let progress_entry = semantic::required(parser, &mut block, "progress", "text highlight")?;
    let progress = parameter::scalar(parser, &progress_entry)?;
    semantic::finish(parser, block, "text highlight");
    Some(TextHighlightDecl {
        fill,
        progress,
        span: entry.span,
    })
}

fn transform(parser: &mut Parser, entry: &SemanticEntry) -> Option<TextAnimationTransformDecl> {
    let mut block = semantic::nested(parser, entry, "text animation transform")?;
    let position = point(parser, &mut block, "position")?;
    let scale = vector(parser, &mut block, "scale")?;
    let rotation = scalar(parser, &mut block, "rotation")?;
    semantic::finish(parser, block, "text animation transform");
    Some(TextAnimationTransformDecl {
        position,
        scale,
        rotation,
    })
}

fn scalar(
    parser: &mut Parser,
    block: &mut crate::authoring::SemanticBlock,
    name: &'static str,
) -> Option<Option<crate::authoring::ParameterDecl<crate::authoring::NumberLiteral>>> {
    match semantic::take(parser, block, name) {
        Some(entry) => parameter::scalar(parser, &entry).map(Some),
        None => Some(None),
    }
}

fn vector(
    parser: &mut Parser,
    block: &mut crate::authoring::SemanticBlock,
    name: &'static str,
) -> Option<Option<crate::authoring::ParameterDecl<crate::authoring::VectorDecl>>> {
    match semantic::take(parser, block, name) {
        Some(entry) => parameter::vector(parser, &entry).map(Some),
        None => Some(None),
    }
}

fn point(
    parser: &mut Parser,
    block: &mut crate::authoring::SemanticBlock,
    name: &'static str,
) -> Option<Option<crate::authoring::ParameterDecl<crate::authoring::PointDecl>>> {
    match semantic::take(parser, block, name) {
        Some(entry) => parameter::point(parser, &entry).map(Some),
        None => Some(None),
    }
}
