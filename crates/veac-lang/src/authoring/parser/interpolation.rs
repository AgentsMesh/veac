use crate::authoring::{
    InterpolationDecl, InterpolationKind, NumberLiteral, SemanticBlock, SemanticEntry,
    SemanticValue,
};

use super::semantic::{finish, number, required};
use super::Parser;

pub fn parse(parser: &mut Parser, entry: &SemanticEntry) -> Option<InterpolationDecl> {
    let [SemanticValue::Identifier(name)] = entry.values.as_slice() else {
        parser.error(
            "AUTHORING_INTERPOLATION",
            "interpolation requires a closed variant".into(),
            entry.span,
        );
        return None;
    };
    let kind = match name.value.as_str() {
        "hold" => leaf(parser, entry, InterpolationKind::Hold),
        "linear" => leaf(parser, entry, InterpolationKind::Linear),
        "ease-in" => leaf(parser, entry, InterpolationKind::EaseIn),
        "ease-out" => leaf(parser, entry, InterpolationKind::EaseOut),
        "ease-in-out" => leaf(parser, entry, InterpolationKind::EaseInOut),
        "cubic-bezier" => entry.block.clone().and_then(|body| cubic(parser, body)),
        "spring" => entry.block.clone().and_then(|body| spring(parser, body)),
        _ => {
            parser.error(
                "AUTHORING_INTERPOLATION",
                format!("unknown interpolation '{}'", name.value),
                name.span,
            );
            return None;
        }
    }?;
    Some(InterpolationDecl {
        kind,
        span: entry.span,
    })
}

pub fn from_identifier(
    parser: &mut Parser,
    name: crate::authoring::Identifier,
) -> InterpolationDecl {
    let kind = match name.value.as_str() {
        "hold" => InterpolationKind::Hold,
        "linear" => InterpolationKind::Linear,
        "ease-in" => InterpolationKind::EaseIn,
        "ease-out" => InterpolationKind::EaseOut,
        "ease-in-out" => InterpolationKind::EaseInOut,
        _ => {
            parser.error(
                "AUTHORING_INTERPOLATION",
                format!("unknown interpolation '{}'", name.value),
                name.span,
            );
            InterpolationKind::Linear
        }
    };
    InterpolationDecl {
        kind,
        span: name.span,
    }
}

fn leaf(
    parser: &mut Parser,
    entry: &SemanticEntry,
    value: InterpolationKind,
) -> Option<InterpolationKind> {
    if entry.block.is_some() {
        parser.error(
            "AUTHORING_INTERPOLATION",
            "this interpolation does not accept a body".into(),
            entry.span,
        );
        None
    } else {
        Some(value)
    }
}

fn cubic(parser: &mut Parser, mut body: SemanticBlock) -> Option<InterpolationKind> {
    let x1 = field(parser, &mut body, "x1", "cubic-bezier interpolation")?;
    let y1 = field(parser, &mut body, "y1", "cubic-bezier interpolation")?;
    let x2 = field(parser, &mut body, "x2", "cubic-bezier interpolation")?;
    let y2 = field(parser, &mut body, "y2", "cubic-bezier interpolation")?;
    finish(parser, body, "cubic-bezier interpolation");
    Some(InterpolationKind::CubicBezier { x1, y1, x2, y2 })
}

fn spring(parser: &mut Parser, mut body: SemanticBlock) -> Option<InterpolationKind> {
    let context = "spring interpolation";
    let frequency = field(parser, &mut body, "frequency", context)?;
    let decay = field(parser, &mut body, "decay", context)?;
    let initial_velocity = field(parser, &mut body, "initial-velocity", context)?;
    finish(parser, body, context);
    Some(InterpolationKind::Spring {
        frequency,
        decay,
        initial_velocity,
    })
}

fn field(
    parser: &mut Parser,
    body: &mut SemanticBlock,
    name: &'static str,
    context: &'static str,
) -> Option<NumberLiteral> {
    let entry = required(parser, body, name, context)?;
    number(parser, &entry, name)
}
