use crate::authoring::{
    RecordSpan, SemanticBlock, SidechainDynamics, SidechainEndpoints, SidechainRelation,
    SidechainTiming,
};

use super::{relation_endpoint, relation_value, Parser};

pub(super) fn parse(parser: &mut Parser, mut body: SemanticBlock) -> Option<SidechainRelation> {
    let endpoints =
        super::relation::required_block(parser, &mut body, "endpoints", "sidechain relation")
            .and_then(|block| endpoints(parser, block));
    let dynamics =
        super::relation::required_block(parser, &mut body, "dynamics", "sidechain relation")
            .and_then(|block| dynamics(parser, block));
    let timing =
        relation_value::take_block(parser, &mut body, "timing", false, "sidechain relation")
            .and_then(|block| timing(parser, block))
            .unwrap_or(SidechainTiming { active: None });
    relation_value::finish(parser, body, "sidechain relation");
    Some(SidechainRelation {
        endpoints: endpoints?,
        dynamics: dynamics?,
        timing,
    })
}

fn endpoints(parser: &mut Parser, mut body: SemanticBlock) -> Option<SidechainEndpoints> {
    let key = relation_value::take_entry(parser, &mut body, "key", true, "sidechain endpoints")
        .and_then(|entry| relation_endpoint::signal(parser, entry, "sidechain endpoints.key"));
    let target =
        relation_value::take_entry(parser, &mut body, "target", true, "sidechain endpoints")
            .and_then(|entry| relation_endpoint::item(parser, entry, "sidechain endpoints.target"));
    relation_value::finish(parser, body, "sidechain endpoints");
    Some(SidechainEndpoints {
        key: key?,
        target: target?,
    })
}

fn dynamics(parser: &mut Parser, mut body: SemanticBlock) -> Option<SidechainDynamics> {
    let threshold = required_number(parser, &mut body, "threshold", "sidechain dynamics");
    let ratio = required_number(parser, &mut body, "ratio", "sidechain dynamics");
    let attack = required_number(parser, &mut body, "attack", "sidechain dynamics");
    let release = required_number(parser, &mut body, "release", "sidechain dynamics");
    relation_value::finish(parser, body, "sidechain dynamics");
    Some(SidechainDynamics {
        threshold: threshold?,
        ratio: ratio?,
        attack: attack?,
        release: release?,
    })
}

fn timing(parser: &mut Parser, mut body: SemanticBlock) -> Option<SidechainTiming> {
    let active = relation_value::take_block(parser, &mut body, "active", false, "sidechain timing")
        .and_then(|block| active(parser, block));
    relation_value::finish(parser, body, "sidechain timing");
    Some(SidechainTiming { active })
}

fn active(parser: &mut Parser, mut body: SemanticBlock) -> Option<RecordSpan> {
    let span = body.span;
    let at = required_number(parser, &mut body, "at", "sidechain active range");
    let duration = required_number(parser, &mut body, "duration", "sidechain active range");
    relation_value::finish(parser, body, "sidechain active range");
    Some(RecordSpan {
        at: at?,
        duration: duration?,
        span,
    })
}

fn required_number(
    parser: &mut Parser,
    body: &mut SemanticBlock,
    name: &str,
    context: &str,
) -> Option<crate::authoring::NumberLiteral> {
    relation_value::take_entry(parser, body, name, true, context)
        .and_then(|entry| relation_value::number(parser, entry, &format!("{context}.{name}")))
}
