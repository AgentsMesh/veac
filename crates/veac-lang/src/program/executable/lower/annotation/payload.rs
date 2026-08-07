use std::collections::BTreeSet;

use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{AnnotationPayload, FillerSuggestion, LanguageConfidence, Rational};

use super::{malformed, ExecutableLowerError};
use crate::program::executable::lower::{animation, value};

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<AnnotationPayload, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::AnnotationMarker, [label, color]) => Ok(AnnotationPayload::Marker {
            label: value::text(Some(label))?.to_owned(),
            color: marker_color(graph, color)?,
        }),
        (Op::AnnotationLanguage, [scores]) => Ok(AnnotationPayload::Language {
            scores: languages(graph, scores)?,
        }),
        (Op::AnnotationSceneBoundary, [confidence, hard_cut]) => {
            Ok(AnnotationPayload::SceneBoundary {
                confidence: percent(graph, confidence)?,
                hard_cut: value::boolean(Some(hard_cut))?,
            })
        }
        (Op::AnnotationScene, []) => Ok(AnnotationPayload::Scene),
        (Op::AnnotationBeat, [confidence, bar, beat, tempo, meter]) => {
            Ok(AnnotationPayload::Beat {
                confidence: percent(graph, confidence)?,
                bar: u64::try_from(value::integer(Some(bar))?).map_err(|_| malformed())?,
                beat_in_bar: u16::try_from(value::integer(Some(beat))?).map_err(|_| malformed())?,
                tempo: rational(Some(tempo))?,
                meter: u16::try_from(value::integer(Some(meter))?).map_err(|_| malformed())?,
            })
        }
        (Op::AnnotationSilence, [mean, confidence]) => Ok(AnnotationPayload::Silence {
            mean_db: value::finite(Some(mean))?,
            confidence: percent(graph, confidence)?,
        }),
        (Op::AnnotationFiller, [token, confidence, suggestion]) => Ok(AnnotationPayload::Filler {
            token: value::text(Some(token))?.to_owned(),
            confidence: percent(graph, confidence)?,
            suggestion: filler(graph, suggestion)?,
        }),
        (Op::AnnotationHighlight, [score, rationale, evidence]) => {
            Ok(AnnotationPayload::Highlight {
                score: percent(graph, score)?,
                rationale: value::text(Some(rationale))?.to_owned(),
                evidence: text_list(evidence)?,
            })
        }
        (Op::AnnotationReview, [action, rationale, confidence]) => Ok(AnnotationPayload::Review {
            action: review(graph, action)?.to_owned(),
            rationale: value::text(Some(rationale))?.to_owned(),
            confidence: percent(graph, confidence)?,
        }),
        _ => Err(malformed()),
    }
}

fn languages(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Vec<LanguageConfidence>, ExecutableLowerError> {
    let mut result = value::list(Some(source))?
        .iter()
        .map(|source| {
            let values = value::description_operands(graph, source, Op::LanguageConfidence)?;
            let [language, confidence] = values else {
                return Err(malformed());
            };
            Ok(LanguageConfidence {
                language: value::text(Some(language))?.to_owned(),
                confidence: percent(graph, confidence)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let unique = result
        .iter()
        .map(|value| value.language.as_str())
        .collect::<BTreeSet<_>>();
    if unique.len() != result.len() {
        return Err(malformed());
    }
    result.sort_by(|left, right| {
        right
            .confidence
            .total_cmp(&left.confidence)
            .then_with(|| left.language.cmp(&right.language))
    });
    Ok(result)
}

fn marker_color(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<veac_ir::Color>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::MarkerColorNone, []) => Ok(None),
        (Op::MarkerColorPresent, [color]) => Ok(Some(value::color(Some(color))?)),
        _ => Err(malformed()),
    }
}

fn filler(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<FillerSuggestion, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::FillerKeep, []) => Ok(FillerSuggestion::Keep),
        (Op::FillerDelete, []) => Ok(FillerSuggestion::Delete),
        (Op::FillerTighten, []) => Ok(FillerSuggestion::Tighten),
        _ => Err(malformed()),
    }
}

fn review(graph: &FrozenDomainGraph, source: &Value) -> Result<&'static str, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::ReviewKeep, []) => Ok("keep"),
        (Op::ReviewRemove, []) => Ok("remove"),
        _ => Err(malformed()),
    }
}

fn percent(graph: &FrozenDomainGraph, source: &Value) -> Result<f64, ExecutableLowerError> {
    animation::percent_value(graph, Some(source))
}

fn rational(source: Option<&Value>) -> Result<Rational, ExecutableLowerError> {
    let exact = value::exact(source)?;
    Rational::new(
        i64::try_from(exact.numerator()).map_err(|_| malformed())?,
        u32::try_from(exact.denominator()).map_err(|_| malformed())?,
    )
    .map_err(|_| malformed())
}

fn text_list(source: &Value) -> Result<Vec<String>, ExecutableLowerError> {
    value::list(Some(source))?
        .iter()
        .map(|value| Ok(value::text(Some(value))?.to_owned()))
        .collect()
}
