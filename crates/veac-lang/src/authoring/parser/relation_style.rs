use crate::authoring::{
    Diagnostic, FadeColor, SemanticBlock, SemanticEntry, Spanned, TransitionStyle,
};

use super::{relation_value, Parser};

pub(super) fn transition(parser: &mut Parser, body: SemanticBlock) -> Option<TransitionStyle> {
    if body.entries.len() != 1 {
        parser.diagnostic(Diagnostic::new(
            "AUTHORING_RELATION_STYLE_SHAPE",
            "transition style requires exactly one variant",
            body.span,
        ));
        return None;
    }
    let entry = body.entries.into_iter().next()?;
    match entry.name.value.as_str() {
        "dissolve" => unit(parser, entry).then_some(TransitionStyle::Dissolve),
        "fade" => fade(parser, entry),
        "wipe" => super::relation_style_motion::wipe(parser, entry),
        "slide" => super::relation_style_motion::slide(parser, entry),
        "zoom" => super::relation_style_motion::zoom(parser, entry),
        "circle" => super::relation_style_motion::circle(parser, entry),
        "pixelize" => pixelize(parser, entry),
        _ => {
            relation_value::unknown(
                parser,
                "AUTHORING_UNKNOWN_TRANSITION_STYLE",
                "transition style",
                entry.name,
            );
            None
        }
    }
}

pub(super) fn variant_body(
    parser: &mut Parser,
    entry: SemanticEntry,
    label: &str,
) -> Option<SemanticBlock> {
    if !entry.values.is_empty() || entry.block.is_none() {
        parser.diagnostic(Diagnostic::new(
            "AUTHORING_FIELD_TYPE",
            format!("{label} must be a block"),
            entry.span,
        ));
        return None;
    }
    entry.block
}

fn unit(parser: &mut Parser, entry: SemanticEntry) -> bool {
    if entry.values.is_empty() && entry.block.is_none() {
        true
    } else {
        parser.diagnostic(Diagnostic::new(
            "AUTHORING_FIELD_TYPE",
            "dissolve style does not accept a body",
            entry.span,
        ));
        false
    }
}

fn fade(parser: &mut Parser, entry: SemanticEntry) -> Option<TransitionStyle> {
    let mut body = variant_body(parser, entry, "fade style")?;
    let color = relation_value::take_entry(parser, &mut body, "color", true, "fade style")
        .and_then(|entry| relation_value::word(parser, entry, "fade style.color"))
        .and_then(|value| {
            let color = match value.value.as_str() {
                "transparent" => FadeColor::Transparent,
                "black" => FadeColor::Black,
                "white" => FadeColor::White,
                _ => {
                    relation_value::unknown(
                        parser,
                        "AUTHORING_UNKNOWN_FADE_COLOR",
                        "fade color",
                        value,
                    );
                    return None;
                }
            };
            Some(Spanned {
                value: color,
                span: value.span,
            })
        });
    relation_value::finish(parser, body, "fade style");
    Some(TransitionStyle::Fade { color: color? })
}

fn pixelize(parser: &mut Parser, entry: SemanticEntry) -> Option<TransitionStyle> {
    let mut body = variant_body(parser, entry, "pixelize style")?;
    let amount = relation_value::take_entry(parser, &mut body, "amount", true, "pixelize style")
        .and_then(|entry| relation_value::number(parser, entry, "pixelize style.amount"));
    relation_value::finish(parser, body, "pixelize style");
    Some(TransitionStyle::Pixelize { amount: amount? })
}
