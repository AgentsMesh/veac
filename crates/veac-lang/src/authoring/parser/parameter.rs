use super::semantic::{finish, nested, number, required, shape};
use super::Parser;
use crate::authoring::{
    NumberLiteral, ParameterDecl, ParameterKey, PointDecl, RectDecl, SemanticBlock, SemanticEntry,
    SemanticValue, Span, VectorDecl,
};

pub(super) fn scalar(
    parser: &mut Parser,
    entry: &SemanticEntry,
) -> Option<ParameterDecl<NumberLiteral>> {
    if matches!(entry.values.as_slice(), [SemanticValue::Number(_)]) && entry.block.is_none() {
        return number(parser, entry, "parameter").map(ParameterDecl::Constant);
    }
    let value = curve(parser, entry, |parser, entry| {
        number(parser, entry, "key value")
    });
    if value.is_none() {
        shape(
            parser,
            entry,
            "parameter must be a number or curve block".to_owned(),
        );
    }
    value
}

pub(super) fn point(
    parser: &mut Parser,
    entry: &SemanticEntry,
) -> Option<ParameterDecl<PointDecl>> {
    structured(parser, entry, "point", point_value)
}

pub(super) fn vector(
    parser: &mut Parser,
    entry: &SemanticEntry,
) -> Option<ParameterDecl<VectorDecl>> {
    structured(parser, entry, "vector", vector_value)
}

pub(super) fn rect(parser: &mut Parser, entry: &SemanticEntry) -> Option<ParameterDecl<RectDecl>> {
    structured(parser, entry, "rectangle", rect_value)
}

fn structured<T>(
    parser: &mut Parser,
    entry: &SemanticEntry,
    context: &'static str,
    value: fn(&mut Parser, SemanticBlock, Span) -> Option<T>,
) -> Option<ParameterDecl<T>> {
    if entry.values.is_empty() {
        let body = nested(parser, entry, context)?;
        return value(parser, body, entry.span).map(ParameterDecl::Constant);
    }
    curve(parser, entry, |parser, entry| {
        let body = nested(parser, entry, "key value")?;
        value(parser, body, entry.span)
    })
}

fn curve<T>(
    parser: &mut Parser,
    entry: &SemanticEntry,
    value: impl Fn(&mut Parser, &SemanticEntry) -> Option<T>,
) -> Option<ParameterDecl<T>> {
    if !matches!(entry.values.as_slice(), [SemanticValue::Identifier(kind)] if kind.value == "curve")
    {
        return None;
    }
    let block = entry.block.as_ref()?;
    let mut keys = Vec::new();
    for key in &block.entries {
        if key.name.value != "key" {
            parser.error(
                "AUTHORING_CURVE_MEMBER",
                "curve accepts only key declarations".to_owned(),
                key.name.span,
            );
            continue;
        }
        if let Some(key) = parse_key(parser, key, &value) {
            keys.push(key);
        }
    }
    if keys.is_empty() {
        parser.error(
            "AUTHORING_CURVE_EMPTY",
            "curve requires at least one key".to_owned(),
            entry.span,
        );
    }
    Some(ParameterDecl::Curve {
        keys,
        span: entry.span,
    })
}

fn parse_key<T>(
    parser: &mut Parser,
    entry: &SemanticEntry,
    value: &impl Fn(&mut Parser, &SemanticEntry) -> Option<T>,
) -> Option<ParameterKey<T>> {
    let id = match entry.values.as_slice() {
        [SemanticValue::Identifier(value)] => value.clone(),
        _ => {
            parser.error(
                "AUTHORING_CURVE_KEY",
                "key requires one stable identifier".to_owned(),
                entry.span,
            );
            return None;
        }
    };
    let mut body = entry.block.clone()?;
    let at_entry = required(parser, &mut body, "at", "curve key")?;
    let value_entry = required(parser, &mut body, "value", "curve key")?;
    let interpolation_entry = required(parser, &mut body, "interpolation", "curve key")?;
    let at = number(parser, &at_entry, "key at")?;
    let key_value = value(parser, &value_entry)?;
    let interpolation = super::interpolation::parse(parser, &interpolation_entry)?;
    finish(parser, body, "curve key");
    Some(ParameterKey {
        id,
        at,
        value: key_value,
        interpolation,
        span: entry.span,
    })
}

fn point_value(parser: &mut Parser, body: SemanticBlock, span: Span) -> Option<PointDecl> {
    vector_parts(parser, body).map(|(x, y)| PointDecl { x, y, span })
}

fn vector_value(parser: &mut Parser, body: SemanticBlock, span: Span) -> Option<VectorDecl> {
    vector_parts(parser, body).map(|(x, y)| VectorDecl { x, y, span })
}

fn vector_parts(
    parser: &mut Parser,
    mut body: SemanticBlock,
) -> Option<(NumberLiteral, NumberLiteral)> {
    let x_entry = required(parser, &mut body, "x", "vector")?;
    let y_entry = required(parser, &mut body, "y", "vector")?;
    let x = number(parser, &x_entry, "x")?;
    let y = number(parser, &y_entry, "y")?;
    finish(parser, body, "vector");
    Some((x, y))
}

fn rect_value(parser: &mut Parser, mut body: SemanticBlock, span: Span) -> Option<RectDecl> {
    let x = field_number(parser, &mut body, "x")?;
    let y = field_number(parser, &mut body, "y")?;
    let width = field_number(parser, &mut body, "width")?;
    let height = field_number(parser, &mut body, "height")?;
    finish(parser, body, "rectangle");
    Some(RectDecl {
        x,
        y,
        width,
        height,
        span,
    })
}

fn field_number(
    parser: &mut Parser,
    body: &mut SemanticBlock,
    name: &'static str,
) -> Option<NumberLiteral> {
    let entry = required(parser, body, name, "rectangle")?;
    number(parser, &entry, name)
}
