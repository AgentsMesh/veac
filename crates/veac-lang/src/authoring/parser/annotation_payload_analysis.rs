use crate::authoring::{
    FillerPayloadDecl, FillerSuggestionDecl, HighlightPayloadDecl, ReviewActionDecl,
    ReviewPayloadDecl, SemanticBlock, SemanticEntry, SemanticValue, SilencePayloadDecl, Spanned,
};

use super::annotation_fields as fields;
use super::{semantic, Parser};

pub(super) fn silence(parser: &mut Parser, mut block: SemanticBlock) -> Option<SilencePayloadDecl> {
    let mean_entry = fields::required(parser, &mut block, "mean-db")?;
    let mean_db = semantic::number(parser, &mean_entry, "silence mean-db")?;
    let confidence_entry = fields::required(parser, &mut block, "confidence")?;
    let confidence = semantic::number(parser, &confidence_entry, "silence confidence")?;
    fields::finish(parser, block);
    Some(SilencePayloadDecl {
        mean_db,
        confidence,
    })
}

pub(super) fn filler(parser: &mut Parser, mut block: SemanticBlock) -> Option<FillerPayloadDecl> {
    let token_entry = fields::required(parser, &mut block, "token")?;
    let token = fields::string(parser, &token_entry)?;
    let confidence_entry = fields::required(parser, &mut block, "confidence")?;
    let confidence = semantic::number(parser, &confidence_entry, "filler confidence")?;
    let suggestion_entry = fields::required(parser, &mut block, "suggestion")?;
    let suggestion = enum_value(parser, &suggestion_entry, |value| match value {
        "keep" => Some(FillerSuggestionDecl::Keep),
        "delete" => Some(FillerSuggestionDecl::Delete),
        "tighten" => Some(FillerSuggestionDecl::Tighten),
        _ => None,
    })?;
    fields::finish(parser, block);
    Some(FillerPayloadDecl {
        token,
        confidence,
        suggestion,
    })
}

pub(super) fn highlight(
    parser: &mut Parser,
    mut block: SemanticBlock,
) -> Option<HighlightPayloadDecl> {
    let score_entry = fields::required(parser, &mut block, "score")?;
    let score = semantic::number(parser, &score_entry, "highlight score")?;
    let rationale_entry = fields::required(parser, &mut block, "rationale")?;
    let rationale = fields::string(parser, &rationale_entry)?;
    let evidence = fields::take_all(&mut block, "evidence")
        .iter()
        .filter_map(|entry| fields::string(parser, entry))
        .collect();
    fields::finish(parser, block);
    Some(HighlightPayloadDecl {
        score,
        rationale,
        evidence,
    })
}

pub(super) fn review(parser: &mut Parser, mut block: SemanticBlock) -> Option<ReviewPayloadDecl> {
    let action_entry = fields::required(parser, &mut block, "action")?;
    let action = enum_value(parser, &action_entry, |value| match value {
        "keep" => Some(ReviewActionDecl::Keep),
        "remove" => Some(ReviewActionDecl::Remove),
        _ => None,
    })?;
    let rationale_entry = fields::required(parser, &mut block, "rationale")?;
    let rationale = fields::string(parser, &rationale_entry)?;
    let confidence_entry = fields::required(parser, &mut block, "confidence")?;
    let confidence = semantic::number(parser, &confidence_entry, "review confidence")?;
    fields::finish(parser, block);
    Some(ReviewPayloadDecl {
        action,
        rationale,
        confidence,
    })
}

fn enum_value<T>(
    parser: &mut Parser,
    entry: &SemanticEntry,
    parse: impl FnOnce(&str) -> Option<T>,
) -> Option<Spanned<T>> {
    let [SemanticValue::Identifier(value)] = entry.values.as_slice() else {
        fields::shape(parser, entry, "one enum value");
        return None;
    };
    parse(&value.value)
        .map(|parsed| Spanned {
            value: parsed,
            span: value.span,
        })
        .or_else(|| {
            parser.error(
                "AUTHORING_UNKNOWN_ENUM_VALUE",
                format!("unknown value '{}'", value.value),
                value.span,
            );
            None
        })
}
