use crate::*;

pub(super) fn valid(payload: &AnnotationPayload, span: AnnotationSpan) -> bool {
    match payload {
        AnnotationPayload::Marker { label, .. } => text(label),
        AnnotationPayload::Language { scores } => {
            matches!(span, AnnotationSpan::Untimed) && language_scores(scores)
        }
        AnnotationPayload::SceneBoundary { confidence, .. } => {
            matches!(span, AnnotationSpan::Point { .. }) && probability(*confidence)
        }
        AnnotationPayload::Scene => matches!(span, AnnotationSpan::Range { .. }),
        AnnotationPayload::Beat {
            confidence,
            beat_in_bar,
            tempo,
            meter,
            ..
        } => {
            matches!(span, AnnotationSpan::Point { .. })
                && probability(*confidence)
                && *meter > 0
                && *beat_in_bar > 0
                && beat_in_bar <= meter
                && tempo.is_positive()
        }
        AnnotationPayload::Silence {
            mean_db,
            confidence,
        } => range(span) && mean_db.is_finite() && probability(*confidence),
        AnnotationPayload::Filler {
            token, confidence, ..
        } => range(span) && text(token) && probability(*confidence),
        AnnotationPayload::Highlight {
            score,
            rationale,
            evidence,
        } => range(span) && probability(*score) && text(rationale) && sorted_text(evidence),
        AnnotationPayload::Review {
            action,
            rationale,
            confidence,
        } => range(span) && text(action) && text(rationale) && probability(*confidence),
    }
}

fn language_scores(values: &[LanguageConfidence]) -> bool {
    !values.is_empty()
        && values
            .iter()
            .all(|value| language(&value.language) && probability(value.confidence))
        && values.windows(2).all(|pair| {
            pair[0].confidence > pair[1].confidence
                || (pair[0].confidence == pair[1].confidence && pair[0].language < pair[1].language)
        })
}

fn range(span: AnnotationSpan) -> bool {
    matches!(span, AnnotationSpan::Range { .. })
}

fn probability(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

pub(super) fn text(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 4096 && !value.contains('\0')
}

fn language(value: &str) -> bool {
    (2..=35).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn sorted_text(values: &[String]) -> bool {
    values.len() <= 256
        && values.iter().all(|value| text(value))
        && values.windows(2).all(|pair| pair[0] < pair[1])
}
