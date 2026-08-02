use crate::authoring::{
    CardinalDirection, CircleDirection, Diagnostic, Identifier, SemanticEntry, Spanned,
    TransitionStyle, ZoomDirection,
};

use super::{relation_style::variant_body, relation_value, Parser};

pub(super) fn wipe(parser: &mut Parser, entry: SemanticEntry) -> Option<TransitionStyle> {
    let mut body = variant_body(parser, entry, "wipe style")?;
    let direction = cardinal(parser, &mut body, "wipe style")?;
    let angle = required_number(parser, &mut body, "angle", "wipe style")?;
    let softness = required_number(parser, &mut body, "softness", "wipe style")?;
    relation_value::finish(parser, body, "wipe style");
    Some(TransitionStyle::Wipe {
        direction,
        angle,
        softness,
    })
}

pub(super) fn slide(parser: &mut Parser, entry: SemanticEntry) -> Option<TransitionStyle> {
    let mut body = variant_body(parser, entry, "slide style")?;
    let direction = cardinal(parser, &mut body, "slide style")?;
    let amount = required_number(parser, &mut body, "amount", "slide style")?;
    relation_value::finish(parser, body, "slide style");
    Some(TransitionStyle::Slide { direction, amount })
}

pub(super) fn zoom(parser: &mut Parser, entry: SemanticEntry) -> Option<TransitionStyle> {
    let mut body = variant_body(parser, entry, "zoom style")?;
    let value = required_word(parser, &mut body, "direction", "zoom style")?;
    let direction = match value.value.as_str() {
        "in" => ZoomDirection::In,
        "out" => ZoomDirection::Out,
        _ => return unknown_direction(parser, value, "zoom"),
    };
    let amount = required_number(parser, &mut body, "amount", "zoom style")?;
    relation_value::finish(parser, body, "zoom style");
    Some(TransitionStyle::Zoom {
        direction: Spanned {
            value: direction,
            span: value.span,
        },
        amount,
    })
}

pub(super) fn circle(parser: &mut Parser, entry: SemanticEntry) -> Option<TransitionStyle> {
    let mut body = variant_body(parser, entry, "circle style")?;
    let value = required_word(parser, &mut body, "direction", "circle style")?;
    let direction = match value.value.as_str() {
        "open" => CircleDirection::Open,
        "close" => CircleDirection::Close,
        _ => return unknown_direction(parser, value, "circle"),
    };
    let softness = required_number(parser, &mut body, "softness", "circle style")?;
    relation_value::finish(parser, body, "circle style");
    Some(TransitionStyle::Circle {
        direction: Spanned {
            value: direction,
            span: value.span,
        },
        softness,
    })
}

fn cardinal(
    parser: &mut Parser,
    body: &mut crate::authoring::SemanticBlock,
    context: &str,
) -> Option<Spanned<CardinalDirection>> {
    let value = required_word(parser, body, "direction", context)?;
    let direction = match value.value.as_str() {
        "left" => CardinalDirection::Left,
        "right" => CardinalDirection::Right,
        "up" => CardinalDirection::Up,
        "down" => CardinalDirection::Down,
        _ => return unknown_direction(parser, value, "cardinal"),
    };
    Some(Spanned {
        value: direction,
        span: value.span,
    })
}

fn required_word(
    parser: &mut Parser,
    body: &mut crate::authoring::SemanticBlock,
    name: &str,
    context: &str,
) -> Option<Identifier> {
    relation_value::take_entry(parser, body, name, true, context)
        .and_then(|entry| relation_value::word(parser, entry, &format!("{context}.{name}")))
}

fn required_number(
    parser: &mut Parser,
    body: &mut crate::authoring::SemanticBlock,
    name: &str,
    context: &str,
) -> Option<crate::authoring::NumberLiteral> {
    relation_value::take_entry(parser, body, name, true, context)
        .and_then(|entry| relation_value::number(parser, entry, &format!("{context}.{name}")))
}

fn unknown_direction<T>(parser: &mut Parser, value: Identifier, family: &str) -> Option<T> {
    parser.diagnostic(Diagnostic::new(
        "AUTHORING_UNKNOWN_TRANSITION_DIRECTION",
        format!("unknown {family} direction {}", value.value),
        value.span,
    ));
    None
}
