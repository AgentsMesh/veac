use crate::authoring::{
    AnnotationPayloadDecl, BeatPayloadDecl, Identifier, LanguageCandidateDecl, LanguagePayloadDecl,
    MarkerPayloadDecl, SceneBoundaryPayloadDecl, SemanticBlock, SemanticEntry, SemanticValue,
};

use super::annotation_fields as fields;
use super::{annotation_payload_analysis as analysis, semantic, Parser};

pub(super) fn parse(
    parser: &mut Parser,
    kind: &Identifier,
    entry: &SemanticEntry,
) -> Option<AnnotationPayloadDecl> {
    let block = semantic::nested(parser, entry, "annotation payload")?;
    match kind.value.as_str() {
        "marker" => marker(parser, block).map(AnnotationPayloadDecl::Marker),
        "language" => language(parser, block).map(AnnotationPayloadDecl::Language),
        "scene-boundary" => scene_boundary(parser, block).map(AnnotationPayloadDecl::SceneBoundary),
        "scene" => scene(parser, block),
        "beat" => beat(parser, block).map(AnnotationPayloadDecl::Beat),
        "silence" => analysis::silence(parser, block).map(AnnotationPayloadDecl::Silence),
        "filler" => analysis::filler(parser, block).map(AnnotationPayloadDecl::Filler),
        "highlight" => analysis::highlight(parser, block).map(AnnotationPayloadDecl::Highlight),
        "review" => analysis::review(parser, block).map(AnnotationPayloadDecl::Review),
        _ => {
            parser.error(
                "AUTHORING_UNKNOWN_ANNOTATION_KIND",
                format!("unknown annotation kind '{}'", kind.value),
                kind.span,
            );
            None
        }
    }
}

fn marker(parser: &mut Parser, mut block: SemanticBlock) -> Option<MarkerPayloadDecl> {
    let label_entry = fields::required(parser, &mut block, "label")?;
    let label = fields::string(parser, &label_entry)?;
    let color = match semantic::take(parser, &mut block, "color") {
        Some(entry) => Some(fields::color(parser, &entry)?),
        None => None,
    };
    fields::finish(parser, block);
    Some(MarkerPayloadDecl { label, color })
}

fn language(parser: &mut Parser, mut block: SemanticBlock) -> Option<LanguagePayloadDecl> {
    let entries = fields::take_all(&mut block, "candidate");
    if entries.is_empty() {
        parser.error(
            "AUTHORING_MISSING_FIELD",
            "language payload requires at least one candidate".to_owned(),
            block.span,
        );
    }
    let candidates = entries
        .iter()
        .filter_map(|entry| candidate(parser, entry))
        .collect();
    fields::finish(parser, block);
    Some(LanguagePayloadDecl { candidates })
}

fn candidate(parser: &mut Parser, entry: &SemanticEntry) -> Option<LanguageCandidateDecl> {
    let [SemanticValue::String(language)] = entry.values.as_slice() else {
        fields::shape(parser, entry, "a language string and block");
        return None;
    };
    let mut block = fields::header_block(parser, entry, "language candidate")?;
    let confidence_entry = fields::required(parser, &mut block, "confidence")?;
    let confidence = semantic::number(parser, &confidence_entry, "candidate confidence")?;
    fields::finish(parser, block);
    Some(LanguageCandidateDecl {
        language: language.clone(),
        confidence,
        span: entry.span,
    })
}

fn scene_boundary(
    parser: &mut Parser,
    mut block: SemanticBlock,
) -> Option<SceneBoundaryPayloadDecl> {
    let confidence_entry = fields::required(parser, &mut block, "confidence")?;
    let confidence = semantic::number(parser, &confidence_entry, "boundary confidence")?;
    let hard_cut_entry = fields::required(parser, &mut block, "hard-cut")?;
    let hard_cut = semantic::boolean(parser, &hard_cut_entry, "hard-cut")?;
    fields::finish(parser, block);
    Some(SceneBoundaryPayloadDecl {
        confidence,
        hard_cut,
    })
}

fn scene(parser: &mut Parser, block: SemanticBlock) -> Option<AnnotationPayloadDecl> {
    fields::finish(parser, block);
    Some(AnnotationPayloadDecl::Scene)
}

fn beat(parser: &mut Parser, mut block: SemanticBlock) -> Option<BeatPayloadDecl> {
    let confidence_entry = fields::required(parser, &mut block, "confidence")?;
    let confidence = semantic::number(parser, &confidence_entry, "beat confidence")?;
    let result = BeatPayloadDecl {
        confidence,
        bar: required_number(parser, &mut block, "bar")?,
        beat_in_bar: required_number(parser, &mut block, "beat-in-bar")?,
        tempo_bpm: required_number(parser, &mut block, "tempo-bpm")?,
        meter: required_number(parser, &mut block, "meter")?,
    };
    fields::finish(parser, block);
    Some(result)
}

fn required_number(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
) -> Option<crate::authoring::NumberLiteral> {
    let entry = fields::required(parser, block, name)?;
    semantic::number(parser, &entry, name)
}
