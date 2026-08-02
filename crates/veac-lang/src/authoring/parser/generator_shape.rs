use super::generator::{assign, paint_span, required};
use super::{generator_geometry, generator_gradient, Parser};
use crate::authoring::{PaintDecl, ShapeGeneratorDecl, Span, StrokeDecl};

pub(super) fn parse(parser: &mut Parser, start: Span) -> Option<ShapeGeneratorDecl> {
    parser.left_brace()?;
    let mut geometry = None;
    let mut fill = None;
    let mut stroke = None;
    while !parser.at_right_brace() && !parser.at_eof() {
        let field = parser.identifier("shape field")?;
        match field.value.as_str() {
            "geometry" => {
                let value = generator_geometry::parse(parser);
                assign(parser, "shape geometry", &mut geometry, value, field.span);
            }
            "fill" => {
                let value = paint(parser, field.span);
                assign(parser, "shape fill", &mut fill, value, field.span);
            }
            "stroke" => {
                let value = stroke_value(parser, field.span);
                assign(parser, "shape stroke", &mut stroke, value, field.span);
            }
            _ => {
                parser.error(
                    "AUTHORING_GENERATOR_FIELD",
                    format!("unknown shape field `{}`", field.value),
                    field.span,
                );
                parser.recover_declaration();
            }
        }
    }
    let end = parser.right_brace()?;
    let span = start.join(end);
    let geometry = required(parser, "geometry", geometry, span)?;
    if fill.is_none() && stroke.is_none() {
        parser.error(
            "AUTHORING_SHAPE_PAINT",
            "shape requires a fill, a stroke, or both".to_owned(),
            span,
        );
    }
    Some(ShapeGeneratorDecl {
        geometry,
        fill,
        stroke,
        span,
    })
}

fn stroke_value(parser: &mut Parser, start: Span) -> Option<StrokeDecl> {
    let width = parser.number("stroke width")?;
    let paint = paint(parser, start)?;
    Some(StrokeDecl {
        span: start.join(paint_span(&paint)),
        width,
        paint,
    })
}

fn paint(parser: &mut Parser, start: Span) -> Option<PaintDecl> {
    let kind = parser.identifier("paint kind")?;
    match kind.value.as_str() {
        "solid" => {
            let color = parser.color("solid paint")?;
            parser.semicolon()?;
            Some(PaintDecl::Solid(color))
        }
        "gradient" => generator_gradient::parse(parser, start)
            .map(Box::new)
            .map(PaintDecl::Gradient),
        _ => {
            parser.error(
                "AUTHORING_PAINT_KIND",
                format!("unknown paint kind `{}`", kind.value),
                kind.span,
            );
            parser.recover_declaration();
            None
        }
    }
}
