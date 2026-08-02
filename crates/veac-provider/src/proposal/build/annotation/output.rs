use veac_ir::{AnnotationPayload, AnnotationSpan, FillerSuggestion, LanguageConfidence};

use super::Builder;
use crate::{
    AnalysisEvidenceKind as Kind, FillerAction, ProviderOutput, ProviderResult, ReviewDecision,
};

pub(super) fn append(builder: &mut Builder<'_>, output: &ProviderOutput) -> ProviderResult<()> {
    match output {
        ProviderOutput::LanguageDetection(value) => {
            let scores = value
                .languages
                .iter()
                .map(|score| LanguageConfidence {
                    language: score.language.clone(),
                    confidence: score.confidence,
                })
                .collect();
            builder.push(
                "language",
                Kind::Language,
                0,
                AnnotationSpan::Untimed,
                AnnotationPayload::Language { scores },
            )
        }
        ProviderOutput::SceneDetection(value) => scenes(builder, value),
        ProviderOutput::BeatDetection(value) => beats(builder, value),
        ProviderOutput::SilenceDetection(value) => silences(builder, value),
        ProviderOutput::FillerDetection(value) => fillers(builder, value),
        ProviderOutput::HighlightDetection(value) => highlights(builder, value),
        _ => unreachable!("annotation builder received another provider output"),
    }
}

fn scenes(builder: &mut Builder<'_>, value: &crate::SceneDetectionResult) -> ProviderResult<()> {
    for (index, boundary) in value.boundaries.iter().enumerate() {
        builder.push(
            "scene-boundary",
            Kind::SceneBoundary,
            index,
            builder.point(boundary.time)?,
            AnnotationPayload::SceneBoundary {
                confidence: boundary.confidence,
                hard_cut: boundary.hard_cut,
            },
        )?;
    }
    for (index, scene) in value.scenes.iter().enumerate() {
        builder.push(
            "scene",
            Kind::Scene,
            index,
            builder.range(*scene)?,
            AnnotationPayload::Scene,
        )?;
    }
    Ok(())
}

fn beats(builder: &mut Builder<'_>, value: &crate::BeatDetectionResult) -> ProviderResult<()> {
    for (index, beat) in value.beats.iter().enumerate() {
        builder.push(
            "beat",
            Kind::Beat,
            index,
            builder.point(beat.time)?,
            AnnotationPayload::Beat {
                confidence: beat.confidence,
                bar: beat.bar,
                beat_in_bar: beat.beat_in_bar,
                tempo: value.tempo,
                meter: value.meter,
            },
        )?;
    }
    Ok(())
}

fn silences(
    builder: &mut Builder<'_>,
    value: &crate::SilenceDetectionResult,
) -> ProviderResult<()> {
    for (index, silence) in value.intervals.iter().enumerate() {
        builder.push(
            "silence",
            Kind::Silence,
            index,
            builder.range(silence.range)?,
            AnnotationPayload::Silence {
                mean_db: silence.mean_db,
                confidence: silence.confidence,
            },
        )?;
    }
    Ok(())
}

fn fillers(builder: &mut Builder<'_>, value: &crate::FillerDetectionResult) -> ProviderResult<()> {
    for (index, occurrence) in value.occurrences.iter().enumerate() {
        let suggestion = match occurrence.suggested_action {
            FillerAction::Keep => FillerSuggestion::Keep,
            FillerAction::Delete => FillerSuggestion::Delete,
            FillerAction::Tighten => FillerSuggestion::Tighten,
        };
        builder.push(
            "filler",
            Kind::Filler,
            index,
            builder.range(occurrence.range)?,
            AnnotationPayload::Filler {
                token: occurrence.token.clone(),
                confidence: occurrence.confidence,
                suggestion,
            },
        )?;
    }
    decisions(
        builder,
        &value.decisions,
        "filler-decision",
        Kind::FillerDecision,
    )
}

fn highlights(
    builder: &mut Builder<'_>,
    value: &crate::HighlightDetectionResult,
) -> ProviderResult<()> {
    for (index, highlight) in value.highlights.iter().enumerate() {
        builder.push(
            "highlight",
            Kind::Highlight,
            index,
            builder.range(highlight.range)?,
            AnnotationPayload::Highlight {
                score: highlight.score,
                rationale: highlight.rationale.clone(),
                evidence: highlight.evidence.clone(),
            },
        )?;
    }
    decisions(
        builder,
        &value.decisions,
        "highlight-decision",
        Kind::HighlightDecision,
    )
}

fn decisions(
    builder: &mut Builder<'_>,
    values: &[ReviewDecision],
    role: &str,
    kind: Kind,
) -> ProviderResult<()> {
    for (index, value) in values.iter().enumerate() {
        builder.push(
            role,
            kind,
            index,
            builder.range(value.range)?,
            AnnotationPayload::Review {
                action: value.action.clone(),
                rationale: value.rationale.clone(),
                confidence: value.confidence,
            },
        )?;
    }
    Ok(())
}
