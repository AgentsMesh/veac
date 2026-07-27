use veac_ir::{
    AnnotationId, AnnotationSpan, AnnotationTarget, EditOperation, RationalTime, TimeRange,
};

use crate::{
    AnalysisEvidenceKind as Kind, AnnotationApplication, ClipTimeBinding, ProviderEditProposal,
    ProviderOutput,
};

#[path = "annotation/payload.rs"]
mod payload;

pub(super) fn matches(
    kind: Kind,
    index: u32,
    id: &AnnotationId,
    proposal: &ProviderEditProposal,
    operation: &EditOperation,
) -> bool {
    let EditOperation::InsertAnnotation { annotation } = operation else {
        return false;
    };
    let crate::ApplicationContext::AnalysisAnnotations(context) =
        proposal.application_context.as_ref()
    else {
        return false;
    };
    let Ok(index) = usize::try_from(index) else {
        return false;
    };
    expected_id(context, kind, index).is_some_and(|expected| expected == *id)
        && annotation.id == *id
        && annotation.target == context.target
        && expected_span(
            context,
            kind,
            index,
            &proposal.source_output,
            proposal.project_timebase,
        )
        .is_some_and(|span| span == annotation.span)
        && annotation.provenance.as_ref().is_some_and(|value| {
            value.producer == proposal.source_request.provider.annotation_producer()
                && value.request_sha256 == proposal.request_hash.value
                && value.response_sha256 == proposal.response_hash.value
        })
        && payload::matches(kind, index, &proposal.source_output, &annotation.payload)
}

fn expected_id(context: &AnnotationApplication, kind: Kind, index: usize) -> Option<AnnotationId> {
    AnnotationId::new(format!(
        "{}_{}_{:04}",
        context.annotation_id_prefix,
        role(kind),
        index + 1
    ))
    .ok()
}

fn role(kind: Kind) -> &'static str {
    match kind {
        Kind::Language => "language",
        Kind::SceneBoundary => "scene-boundary",
        Kind::Scene => "scene",
        Kind::Beat => "beat",
        Kind::Silence => "silence",
        Kind::Filler => "filler",
        Kind::FillerDecision => "filler-decision",
        Kind::Highlight => "highlight",
        Kind::HighlightDecision => "highlight-decision",
    }
}

fn expected_span(
    context: &AnnotationApplication,
    kind: Kind,
    index: usize,
    source: &ProviderOutput,
    timebase: u32,
) -> Option<AnnotationSpan> {
    match source_span(kind, index, source)? {
        SourceSpan::Untimed if context.time.is_none() => Some(AnnotationSpan::Untimed),
        SourceSpan::Point(value) => Some(AnnotationSpan::Point {
            at: mapped_point(context, value, timebase)?,
        }),
        SourceSpan::Range(value) => {
            let start = mapped_point(context, value.start, timebase)?;
            let duration = super::super::time::convert(value.duration, timebase).ok()?;
            Some(AnnotationSpan::Range {
                range: TimeRange::new(start, duration).ok()?,
            })
        }
        SourceSpan::Untimed => None,
    }
}

fn mapped_point(
    context: &AnnotationApplication,
    value: RationalTime,
    timebase: u32,
) -> Option<RationalTime> {
    if matches!(
        context.target,
        AnnotationTarget::Project | AnnotationTarget::MulticamGroup { .. }
    ) {
        return None;
    }
    let binding = context.time?;
    let mapped = super::super::time::clip_time(
        value,
        ClipTimeBinding {
            provider_origin: binding.provider_origin,
            clip_local_origin: binding.target_origin,
        },
        timebase,
    )
    .ok()?;
    (mapped.value >= 0).then_some(mapped)
}

fn source_span(kind: Kind, index: usize, source: &ProviderOutput) -> Option<SourceSpan> {
    match (kind, source) {
        (Kind::Language, ProviderOutput::LanguageDetection(_)) if index == 0 => {
            Some(SourceSpan::Untimed)
        }
        (Kind::SceneBoundary, ProviderOutput::SceneDetection(value)) => value
            .boundaries
            .get(index)
            .map(|value| SourceSpan::Point(value.time)),
        (Kind::Scene, ProviderOutput::SceneDetection(value)) => {
            value.scenes.get(index).copied().map(SourceSpan::Range)
        }
        (Kind::Beat, ProviderOutput::BeatDetection(value)) => value
            .beats
            .get(index)
            .map(|value| SourceSpan::Point(value.time)),
        (Kind::Silence, ProviderOutput::SilenceDetection(value)) => value
            .intervals
            .get(index)
            .map(|value| SourceSpan::Range(value.range)),
        (Kind::Filler, ProviderOutput::FillerDetection(value)) => value
            .occurrences
            .get(index)
            .map(|value| SourceSpan::Range(value.range)),
        (Kind::FillerDecision, ProviderOutput::FillerDetection(value)) => value
            .decisions
            .get(index)
            .map(|value| SourceSpan::Range(value.range)),
        (Kind::Highlight, ProviderOutput::HighlightDetection(value)) => value
            .highlights
            .get(index)
            .map(|value| SourceSpan::Range(value.range)),
        (Kind::HighlightDecision, ProviderOutput::HighlightDetection(value)) => value
            .decisions
            .get(index)
            .map(|value| SourceSpan::Range(value.range)),
        _ => None,
    }
}

enum SourceSpan {
    Untimed,
    Point(RationalTime),
    Range(TimeRange),
}
