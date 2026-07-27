use super::generator::{assign, required};
use super::Parser;
use crate::authoring::{
    GradientGeneratorDecl, GradientGeometryDecl, GradientStopDecl, NumberLiteral, PointDecl, Span,
};

pub(super) fn parse(parser: &mut Parser, start: Span) -> Option<GradientGeneratorDecl> {
    let kind = parser.identifier("gradient geometry")?;
    parser.left_brace()?;
    match kind.value.as_str() {
        "linear" => linear(parser, start),
        "radial" => radial(parser, start),
        _ => {
            parser.error(
                "AUTHORING_GRADIENT_KIND",
                format!("unknown gradient geometry `{}`", kind.value),
                kind.span,
            );
            parser.recover_declaration();
            None
        }
    }
}

fn linear(parser: &mut Parser, start: Span) -> Option<GradientGeneratorDecl> {
    let mut from = None;
    let mut to = None;
    let mut stops = Vec::new();
    while !parser.at_right_brace() && !parser.at_eof() {
        let field = parser.identifier("linear gradient field")?;
        match field.value.as_str() {
            "from" => {
                let value = point(parser);
                assign(parser, "gradient from", &mut from, value, field.span);
            }
            "to" => {
                let value = point(parser);
                assign(parser, "gradient to", &mut to, value, field.span);
            }
            "stop" => stops.push(stop(parser)?),
            _ => unknown(parser, &field.value, field.span),
        }
    }
    let end = parser.right_brace()?;
    let span = start.join(end);
    let from = required(parser, "from", from, span)?;
    let to = required(parser, "to", to, span)?;
    validate_stops(parser, &stops, span);
    Some(GradientGeneratorDecl {
        geometry: GradientGeometryDecl::Linear { from, to },
        stops,
        span,
    })
}

fn radial(parser: &mut Parser, start: Span) -> Option<GradientGeneratorDecl> {
    let mut center = None;
    let mut radius = None;
    let mut stops = Vec::new();
    while !parser.at_right_brace() && !parser.at_eof() {
        let field = parser.identifier("radial gradient field")?;
        match field.value.as_str() {
            "center" => {
                let value = point(parser);
                assign(parser, "gradient center", &mut center, value, field.span);
            }
            "radius" => {
                let value = number_statement(parser, "gradient radius");
                assign(parser, "gradient radius", &mut radius, value, field.span);
            }
            "stop" => stops.push(stop(parser)?),
            _ => unknown(parser, &field.value, field.span),
        }
    }
    let end = parser.right_brace()?;
    let span = start.join(end);
    let center = required(parser, "center", center, span)?;
    let radius = required(parser, "radius", radius, span)?;
    validate_stops(parser, &stops, span);
    Some(GradientGeneratorDecl {
        geometry: GradientGeometryDecl::Radial { center, radius },
        stops,
        span,
    })
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

fn stop(parser: &mut Parser) -> Option<GradientStopDecl> {
    let position = parser.number("gradient stop position")?;
    let color = parser.color("gradient stop")?;
    parser.semicolon()?;
    Some(GradientStopDecl { position, color })
}

fn validate_stops(parser: &mut Parser, stops: &[GradientStopDecl], span: Span) {
    if stops.len() < 2 {
        parser.error(
            "AUTHORING_GRADIENT_STOPS",
            "gradient requires at least two ordered stops".to_owned(),
            span,
        );
    }
}

fn unknown(parser: &mut Parser, name: &str, span: Span) {
    parser.error(
        "AUTHORING_GENERATOR_FIELD",
        format!("unknown gradient field `{name}`"),
        span,
    );
    parser.recover_declaration();
}
