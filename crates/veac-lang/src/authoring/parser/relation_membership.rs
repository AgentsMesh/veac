use crate::authoring::{AvLinkRelation, Diagnostic, GroupRelation, ItemEndpoint, SemanticBlock};

use super::{relation_endpoint, relation_value, Parser};

pub(super) fn group(parser: &mut Parser, mut body: SemanticBlock) -> Option<GroupRelation> {
    let members = super::relation::required_block(parser, &mut body, "members", "group relation")
        .and_then(|block| members(parser, block, "group", 2));
    relation_value::finish(parser, body, "group relation");
    Some(GroupRelation { members: members? })
}

pub(super) fn av_link(parser: &mut Parser, mut body: SemanticBlock) -> Option<AvLinkRelation> {
    let endpoints =
        super::relation::required_block(parser, &mut body, "endpoints", "av-link relation");
    relation_value::finish(parser, body, "av-link relation");
    let mut endpoints = endpoints?;
    let video =
        relation_value::take_entry(parser, &mut endpoints, "video", true, "av-link endpoints")
            .and_then(|entry| relation_endpoint::item(parser, entry, "av-link endpoints.video"));
    let audio =
        relation_value::take_block(parser, &mut endpoints, "audio", true, "av-link endpoints")
            .and_then(|block| members(parser, block, "av-link audio", 1));
    relation_value::finish(parser, endpoints, "av-link endpoints");
    Some(AvLinkRelation {
        video: video?,
        audio: audio?,
    })
}

fn members(
    parser: &mut Parser,
    body: SemanticBlock,
    label: &str,
    minimum: usize,
) -> Option<Vec<ItemEndpoint>> {
    let span = body.span;
    let members: Vec<_> = body
        .entries
        .into_iter()
        .filter_map(|entry| relation_endpoint::member(parser, entry, label))
        .collect();
    if members.len() < minimum {
        parser.diagnostic(Diagnostic::new(
            "AUTHORING_RELATION_MEMBER_COUNT",
            format!("{label} requires at least {minimum} item member(s)"),
            span,
        ));
        return None;
    }
    Some(members)
}
