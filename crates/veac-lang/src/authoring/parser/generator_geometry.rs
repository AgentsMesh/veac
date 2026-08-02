use super::generator::{assign, required};
use super::Parser;
use crate::authoring::{
    BoundsDecl, NumberLiteral, PathCommandDecl, PointDecl, ShapeGeometryDecl, Span,
};

pub(super) fn parse(parser: &mut Parser) -> Option<ShapeGeometryDecl> {
    let kind = parser.identifier("shape geometry")?;
    parser.left_brace()?;
    match kind.value.as_str() {
        "rectangle" => rectangle(parser, false, kind.span),
        "rounded-rectangle" => rectangle(parser, true, kind.span),
        "ellipse" => ellipse(parser, kind.span),
        "polygon" => polygon(parser, kind.span),
        "path" => path(parser),
        _ => {
            parser.error(
                "AUTHORING_SHAPE_GEOMETRY",
                format!("unknown shape geometry `{}`", kind.value),
                kind.span,
            );
            parser.recover_declaration();
            None
        }
    }
}

fn rectangle(parser: &mut Parser, rounded: bool, start: Span) -> Option<ShapeGeometryDecl> {
    let mut bounds = None;
    let mut radius = None;
    while !parser.at_right_brace() && !parser.at_eof() {
        let field = parser.identifier("rectangle field")?;
        match field.value.as_str() {
            "bounds" => {
                let value = bounds_value(parser);
                assign(parser, "bounds", &mut bounds, value, field.span);
            }
            "radius" if rounded => {
                let value = number_statement(parser, "corner radius");
                assign(parser, "radius", &mut radius, value, field.span);
            }
            _ => unknown(parser, &field.value, field.span),
        }
    }
    let end = parser.right_brace()?;
    let span = start.join(end);
    let bounds = required(parser, "bounds", bounds, span)?;
    if rounded {
        let radius = required(parser, "radius", radius, span)?;
        Some(ShapeGeometryDecl::RoundedRectangle { bounds, radius })
    } else {
        Some(ShapeGeometryDecl::Rectangle(bounds))
    }
}

fn ellipse(parser: &mut Parser, start: Span) -> Option<ShapeGeometryDecl> {
    let mut bounds = None;
    while !parser.at_right_brace() && !parser.at_eof() {
        let field = parser.identifier("ellipse field")?;
        if field.value == "bounds" {
            let value = bounds_value(parser);
            assign(parser, "bounds", &mut bounds, value, field.span);
        } else {
            unknown(parser, &field.value, field.span);
        }
    }
    let end = parser.right_brace()?;
    required(parser, "bounds", bounds, start.join(end)).map(ShapeGeometryDecl::Ellipse)
}

fn polygon(parser: &mut Parser, start: Span) -> Option<ShapeGeometryDecl> {
    let mut points = Vec::new();
    while !parser.at_right_brace() && !parser.at_eof() {
        let field = parser.identifier("polygon field")?;
        if field.value == "point" {
            points.push(point(parser)?);
        } else {
            unknown(parser, &field.value, field.span);
        }
    }
    let end = parser.right_brace()?;
    if points.len() < 3 {
        parser.error(
            "AUTHORING_POLYGON_POINTS",
            "polygon requires at least three points".to_owned(),
            start.join(end),
        );
    }
    Some(ShapeGeometryDecl::Polygon(points))
}

fn path(parser: &mut Parser) -> Option<ShapeGeometryDecl> {
    let mut commands = Vec::new();
    while !parser.at_right_brace() && !parser.at_eof() {
        let command = parser.identifier("path command")?;
        match command.value.as_str() {
            "move" => commands.push(PathCommandDecl::Move(point(parser)?)),
            "line" => commands.push(PathCommandDecl::Line(point(parser)?)),
            "close" => {
                let end = parser.semicolon()?;
                commands.push(PathCommandDecl::Close(command.span.join(end)));
            }
            _ => unknown(parser, &command.value, command.span),
        }
    }
    parser.right_brace()?;
    Some(ShapeGeometryDecl::Path(commands))
}

fn bounds_value(parser: &mut Parser) -> Option<BoundsDecl> {
    let value = BoundsDecl {
        x: parser.number("bounds x")?,
        y: parser.number("bounds y")?,
        width: parser.number("bounds width")?,
        height: parser.number("bounds height")?,
    };
    parser.semicolon()?;
    Some(value)
}

fn point(parser: &mut Parser) -> Option<PointDecl> {
    let x = parser.number("point x")?;
    let y = parser.number("point y")?;
    let span = x.span.join(y.span);
    parser.semicolon()?;
    Some(PointDecl { x, y, span })
}

fn number_statement(parser: &mut Parser, context: &'static str) -> Option<NumberLiteral> {
    let value = parser.number(context)?;
    parser.semicolon()?;
    Some(value)
}

fn unknown(parser: &mut Parser, name: &str, span: Span) {
    parser.error(
        "AUTHORING_GENERATOR_FIELD",
        format!("unknown geometry field `{name}`"),
        span,
    );
    parser.recover_declaration();
}
