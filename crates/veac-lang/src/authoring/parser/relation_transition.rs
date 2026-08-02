use crate::authoring::{
    Diagnostic, SemanticBlock, Spanned, TransitionAlignment, TransitionEndpoints,
    TransitionRelation, TransitionTiming,
};

use super::{relation_endpoint, relation_value, Parser};

pub(super) fn parse(parser: &mut Parser, mut body: SemanticBlock) -> Option<TransitionRelation> {
    let endpoints =
        super::relation::required_block(parser, &mut body, "endpoints", "transition relation")
            .and_then(|block| endpoints(parser, block));
    let timing =
        super::relation::required_block(parser, &mut body, "timing", "transition relation")
            .and_then(|block| timing(parser, block));
    let style = super::relation::required_block(parser, &mut body, "style", "transition relation")
        .and_then(|block| super::relation_style::transition(parser, block));
    relation_value::finish(parser, body, "transition relation");
    Some(TransitionRelation {
        endpoints: endpoints?,
        timing: timing?,
        style: style?,
    })
}

fn endpoints(parser: &mut Parser, mut body: SemanticBlock) -> Option<TransitionEndpoints> {
    let from = relation_value::take_entry(parser, &mut body, "from", true, "transition endpoints")
        .and_then(|entry| relation_endpoint::item(parser, entry, "transition endpoints.from"));
    let to = relation_value::take_entry(parser, &mut body, "to", true, "transition endpoints")
        .and_then(|entry| relation_endpoint::item(parser, entry, "transition endpoints.to"));
    relation_value::finish(parser, body, "transition endpoints");
    Some(TransitionEndpoints {
        from: from?,
        to: to?,
    })
}

fn timing(parser: &mut Parser, mut body: SemanticBlock) -> Option<TransitionTiming> {
    let duration =
        relation_value::take_entry(parser, &mut body, "duration", true, "transition timing")
            .and_then(|entry| relation_value::number(parser, entry, "transition timing.duration"));
    let alignment =
        relation_value::take_entry(parser, &mut body, "alignment", true, "transition timing")
            .and_then(|entry| relation_value::word(parser, entry, "transition timing.alignment"))
            .and_then(|value| alignment(parser, value));
    relation_value::finish(parser, body, "transition timing");
    Some(TransitionTiming {
        duration: duration?,
        alignment: alignment?,
    })
}

fn alignment(
    parser: &mut Parser,
    value: crate::authoring::Identifier,
) -> Option<Spanned<TransitionAlignment>> {
    let alignment = match value.value.as_str() {
        "before-cut" => TransitionAlignment::BeforeCut,
        "centered" => TransitionAlignment::Centered,
        "after-cut" => TransitionAlignment::AfterCut,
        _ => {
            parser.diagnostic(Diagnostic::new(
                "AUTHORING_UNKNOWN_TRANSITION_ALIGNMENT",
                format!("unknown transition alignment {}", value.value),
                value.span,
            ));
            return None;
        }
    };
    Some(Spanned {
        value: alignment,
        span: value.span,
    })
}
