use veac_ir::{AnnotationPayload, FillerSuggestion};

use crate::{AnalysisEvidenceKind as Kind, FillerAction, ProviderOutput, ReviewDecision};

pub(super) fn matches(
    kind: Kind,
    index: usize,
    source: &ProviderOutput,
    payload: &AnnotationPayload,
) -> bool {
    match (kind, source, payload) {
        (
            Kind::Language,
            ProviderOutput::LanguageDetection(value),
            AnnotationPayload::Language { scores },
        ) => {
            index == 0
                && scores.len() == value.languages.len()
                && scores
                    .iter()
                    .zip(&value.languages)
                    .all(|(actual, expected)| {
                        actual.language == expected.language
                            && actual.confidence == expected.confidence
                    })
        }
        (
            Kind::SceneBoundary,
            ProviderOutput::SceneDetection(value),
            AnnotationPayload::SceneBoundary {
                confidence,
                hard_cut,
            },
        ) => value.boundaries.get(index).is_some_and(|expected| {
            *confidence == expected.confidence && *hard_cut == expected.hard_cut
        }),
        (Kind::Scene, ProviderOutput::SceneDetection(value), AnnotationPayload::Scene) => {
            index < value.scenes.len()
        }
        (
            Kind::Beat,
            ProviderOutput::BeatDetection(value),
            AnnotationPayload::Beat {
                confidence,
                bar,
                beat_in_bar,
                tempo,
                meter,
            },
        ) => value.beats.get(index).is_some_and(|expected| {
            *confidence == expected.confidence
                && *bar == expected.bar
                && *beat_in_bar == expected.beat_in_bar
                && *tempo == value.tempo
                && *meter == value.meter
        }),
        (
            Kind::Silence,
            ProviderOutput::SilenceDetection(value),
            AnnotationPayload::Silence {
                mean_db,
                confidence,
            },
        ) => value.intervals.get(index).is_some_and(|expected| {
            *mean_db == expected.mean_db && *confidence == expected.confidence
        }),
        (
            Kind::Filler,
            ProviderOutput::FillerDetection(value),
            AnnotationPayload::Filler {
                token,
                confidence,
                suggestion,
            },
        ) => value.occurrences.get(index).is_some_and(|expected| {
            token == &expected.token
                && *confidence == expected.confidence
                && *suggestion == filler(expected.suggested_action)
        }),
        (
            Kind::FillerDecision,
            ProviderOutput::FillerDetection(value),
            AnnotationPayload::Review {
                action,
                rationale,
                confidence,
            },
        ) => decision(value.decisions.get(index), action, rationale, *confidence),
        (
            Kind::Highlight,
            ProviderOutput::HighlightDetection(value),
            AnnotationPayload::Highlight {
                score,
                rationale,
                evidence,
            },
        ) => value.highlights.get(index).is_some_and(|expected| {
            *score == expected.score
                && rationale == &expected.rationale
                && evidence == &expected.evidence
        }),
        (
            Kind::HighlightDecision,
            ProviderOutput::HighlightDetection(value),
            AnnotationPayload::Review {
                action,
                rationale,
                confidence,
            },
        ) => decision(value.decisions.get(index), action, rationale, *confidence),
        _ => false,
    }
}

fn decision(
    expected: Option<&ReviewDecision>,
    action: &str,
    rationale: &str,
    confidence: f64,
) -> bool {
    expected.is_some_and(|expected| {
        action == expected.action
            && rationale == expected.rationale
            && confidence == expected.confidence
    })
}

fn filler(value: FillerAction) -> FillerSuggestion {
    match value {
        FillerAction::Keep => FillerSuggestion::Keep,
        FillerAction::Delete => FillerSuggestion::Delete,
        FillerAction::Tighten => FillerSuggestion::Tighten,
    }
}
