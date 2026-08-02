use crate::authoring::{AnnotationPayloadDecl, FillerSuggestionDecl, ReviewActionDecl};
use veac_ir::{AnnotationPayload, FillerSuggestion, LanguageConfidence};

use super::context::Context;
use super::{color, integer, value};

pub(super) fn lower(
    ctx: &mut Context,
    declaration: &AnnotationPayloadDecl,
) -> Option<AnnotationPayload> {
    Some(match declaration {
        AnnotationPayloadDecl::Marker(value) => AnnotationPayload::Marker {
            label: value.label.value.clone(),
            color: match &value.color {
                Some(value) => Some(color::lower(ctx, value)?),
                None => None,
            },
        },
        AnnotationPayloadDecl::Language(value) => AnnotationPayload::Language {
            scores: {
                let mut scores = value
                    .candidates
                    .iter()
                    .map(|candidate| {
                        Some(LanguageConfidence {
                            language: candidate.language.value.clone(),
                            confidence: value::unitless(ctx, &candidate.confidence)?,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                scores.sort_by(|left, right| {
                    right
                        .confidence
                        .total_cmp(&left.confidence)
                        .then_with(|| left.language.cmp(&right.language))
                });
                scores
            },
        },
        AnnotationPayloadDecl::SceneBoundary(value) => AnnotationPayload::SceneBoundary {
            confidence: value::unitless(ctx, &value.confidence)?,
            hard_cut: value.hard_cut.value,
        },
        AnnotationPayloadDecl::Scene => AnnotationPayload::Scene,
        AnnotationPayloadDecl::Beat(value) => AnnotationPayload::Beat {
            confidence: value::unitless(ctx, &value.confidence)?,
            bar: integer::u64(ctx, &value.bar, "beat bar")?,
            beat_in_bar: integer::u16(ctx, &value.beat_in_bar, "beat-in-bar")?,
            tempo: value::rational(
                value::unitless(ctx, &value.tempo_bpm)?,
                value.tempo_bpm.span,
                ctx,
            )?,
            meter: integer::u16(ctx, &value.meter, "beat meter")?,
        },
        AnnotationPayloadDecl::Silence(value) => AnnotationPayload::Silence {
            mean_db: value::scalar(ctx, &value.mean_db, "db")?,
            confidence: value::unitless(ctx, &value.confidence)?,
        },
        AnnotationPayloadDecl::Filler(value) => AnnotationPayload::Filler {
            token: value.token.value.clone(),
            confidence: value::unitless(ctx, &value.confidence)?,
            suggestion: match value.suggestion.value {
                FillerSuggestionDecl::Keep => FillerSuggestion::Keep,
                FillerSuggestionDecl::Delete => FillerSuggestion::Delete,
                FillerSuggestionDecl::Tighten => FillerSuggestion::Tighten,
            },
        },
        AnnotationPayloadDecl::Highlight(value) => highlight(ctx, value)?,
        AnnotationPayloadDecl::Review(value) => AnnotationPayload::Review {
            action: match value.action.value {
                ReviewActionDecl::Keep => "keep".to_owned(),
                ReviewActionDecl::Remove => "remove".to_owned(),
            },
            rationale: value.rationale.value.clone(),
            confidence: value::unitless(ctx, &value.confidence)?,
        },
    })
}

fn highlight(
    ctx: &mut Context,
    value: &crate::authoring::HighlightPayloadDecl,
) -> Option<AnnotationPayload> {
    let mut evidence = value
        .evidence
        .iter()
        .map(|value| value.value.clone())
        .collect::<Vec<_>>();
    evidence.sort();
    Some(AnnotationPayload::Highlight {
        score: value::unitless(ctx, &value.score)?,
        rationale: value.rationale.value.clone(),
        evidence,
    })
}
