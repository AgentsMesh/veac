use crate::authoring::{
    AnnotationProvenanceDecl, AnnotationTargetDecl, AnnotationTimingDecl, SemanticEntry,
    SemanticValue,
};

use super::annotation_fields as fields;
use super::{semantic, Parser};

pub(super) fn target(parser: &mut Parser, entry: &SemanticEntry) -> Option<AnnotationTargetDecl> {
    match entry.values.as_slice() {
        [SemanticValue::Identifier(kind)] if kind.value == "project" && entry.block.is_none() => {
            Some(AnnotationTargetDecl::Project { span: entry.span })
        }
        [SemanticValue::Identifier(kind), SemanticValue::Identifier(id)]
            if entry.block.is_none() =>
        {
            Some(match kind.value.as_str() {
                "sequence" => AnnotationTargetDecl::Sequence(id.clone()),
                "layer" => AnnotationTargetDecl::Layer(id.clone()),
                "item" => AnnotationTargetDecl::Item(id.clone()),
                "resource" => AnnotationTargetDecl::Resource(id.clone()),
                "multicam" => AnnotationTargetDecl::Multicam(id.clone()),
                _ => return invalid_target(parser, entry),
            })
        }
        _ => invalid_target(parser, entry),
    }
}

pub(super) fn timing(parser: &mut Parser, entry: &SemanticEntry) -> Option<AnnotationTimingDecl> {
    let [SemanticValue::Identifier(kind)] = entry.values.as_slice() else {
        fields::shape(parser, entry, "a span kind and optional block");
        return None;
    };
    match kind.value.as_str() {
        "untimed" if entry.block.is_none() => {
            Some(AnnotationTimingDecl::Untimed { span: entry.span })
        }
        "point" => {
            let mut block = fields::header_block(parser, entry, "annotation point")?;
            let at_entry = fields::required(parser, &mut block, "at")?;
            let at = semantic::number(parser, &at_entry, "annotation point at")?;
            fields::finish(parser, block);
            Some(AnnotationTimingDecl::Point {
                at,
                span: entry.span,
            })
        }
        "range" => range(parser, entry),
        _ => {
            fields::shape(parser, entry, "untimed, point { ... }, or range { ... }");
            None
        }
    }
}

fn range(parser: &mut Parser, entry: &SemanticEntry) -> Option<AnnotationTimingDecl> {
    let mut block = fields::header_block(parser, entry, "annotation range")?;
    let at_entry = fields::required(parser, &mut block, "at")?;
    let at = semantic::number(parser, &at_entry, "annotation range at")?;
    let duration_entry = fields::required(parser, &mut block, "duration")?;
    let duration = semantic::number(parser, &duration_entry, "annotation range duration")?;
    fields::finish(parser, block);
    Some(AnnotationTimingDecl::Range {
        at,
        duration,
        span: entry.span,
    })
}

pub(super) fn provenance(
    parser: &mut Parser,
    entry: &SemanticEntry,
) -> Option<AnnotationProvenanceDecl> {
    let mut block = semantic::nested(parser, entry, "annotation provenance")?;
    let producer_entry = fields::required(parser, &mut block, "producer")?;
    let producer = fields::string(parser, &producer_entry)?;
    let request_entry = fields::required(parser, &mut block, "request-sha256")?;
    let request_sha256 = fields::string(parser, &request_entry)?;
    let response_entry = fields::required(parser, &mut block, "response-sha256")?;
    let response_sha256 = fields::string(parser, &response_entry)?;
    fields::finish(parser, block);
    Some(AnnotationProvenanceDecl {
        producer,
        request_sha256,
        response_sha256,
        span: entry.span,
    })
}

fn invalid_target<T>(parser: &mut Parser, entry: &SemanticEntry) -> Option<T> {
    parser.error(
        "AUTHORING_INVALID_ANNOTATION_TARGET",
        "target expects project or a typed sequence/layer/item/resource/multicam reference"
            .to_owned(),
        entry.span,
    );
    None
}
