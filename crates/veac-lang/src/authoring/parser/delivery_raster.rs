use crate::authoring::{
    CaptionOutput, DeliveryRasterDecl, NumberLiteral, OutputKeyword, SemanticBlock, SemanticEntry,
    SemanticValue,
};

use super::semantic::{finish, number, required, word};
use super::Parser;

pub(super) fn parse(parser: &mut Parser, mut block: SemanticBlock) -> Option<DeliveryRasterDecl> {
    let canvas = required(parser, &mut block, "canvas", "delivery raster")?;
    let (width, height) = canvas_size(parser, &canvas)?;
    let frame_rate = required(parser, &mut block, "frame-rate", "delivery raster")
        .and_then(|entry| number(parser, &entry, "delivery frame-rate"))?;
    let captions = required(parser, &mut block, "captions", "delivery raster")
        .and_then(|entry| word(parser, &entry, "delivery captions"))
        .and_then(|value| match CaptionOutput::parse(&value.value) {
            Some(value) => Some(value),
            None => {
                parser.error(
                    "AUTHORING_OUTPUT_ENUM",
                    format!("invalid captions value '{}'", value.value),
                    value.span,
                );
                None
            }
        })?;
    let span = block.span;
    finish(parser, block, "delivery raster");
    Some(DeliveryRasterDecl {
        width,
        height,
        frame_rate,
        captions,
        span,
    })
}

fn canvas_size(
    parser: &mut Parser,
    entry: &SemanticEntry,
) -> Option<(NumberLiteral, NumberLiteral)> {
    let values = entry.values.as_slice();
    if entry.block.is_none() {
        if let [SemanticValue::Number(width), SemanticValue::Identifier(by), SemanticValue::Number(height)] =
            values
        {
            if by.value == "by" {
                return Some((width.clone(), height.clone()));
            }
        }
    }
    super::semantic::shape(
        parser,
        entry,
        "delivery canvas must be `<width> by <height>`".to_owned(),
    );
    None
}
