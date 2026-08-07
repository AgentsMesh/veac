use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult, ProducerFingerprint};

use super::{
    AnalysisDescriptor, AnalysisResult, AnalysisResultEnvelope, BeatMarker, SceneBoundary,
    ANALYSIS_RESULT_CONTRACT_VERSION, ANALYSIS_RESULT_SCHEMA_ID,
};

const MAX_ANALYSIS_BOUNDARIES: usize = 1_000_000;
const CONFIDENCE_SCALE: u32 = 1_000_000;

pub(super) fn envelope(value: &AnalysisResultEnvelope) -> ArtifactResult<()> {
    if value.schema != ANALYSIS_RESULT_SCHEMA_ID
        || value.schema_version != ANALYSIS_RESULT_CONTRACT_VERSION
    {
        return invalid("unsupported analysis result contract");
    }
    descriptor(&value.descriptor)?;
    match (&value.descriptor, &value.result) {
        (AnalysisDescriptor::SceneBoundaries(_), AnalysisResult::SceneBoundaries(result)) => {
            timestamps(result.boundaries.iter().map(SceneBoundary::values))?;
        }
        (AnalysisDescriptor::BeatMarkers(_), AnalysisResult::BeatMarkers(result)) => {
            timestamps(result.markers.iter().map(BeatMarker::values))?;
        }
        _ => return invalid("analysis descriptor and result types do not match"),
    }
    Ok(())
}

pub(super) fn descriptor(value: &AnalysisDescriptor) -> ArtifactResult<()> {
    match value {
        AnalysisDescriptor::SceneBoundaries(value)
            if value.sensitivity_millionths <= CONFIDENCE_SCALE =>
        {
            Ok(())
        }
        AnalysisDescriptor::SceneBoundaries(_) => {
            invalid("analysis sensitivity exceeds its fixed-point range")
        }
        AnalysisDescriptor::BeatMarkers(value)
            if value.minimum_bpm_milli > 0
                && value.minimum_bpm_milli <= value.maximum_bpm_milli
                && value.maximum_bpm_milli <= 1_000_000 =>
        {
            Ok(())
        }
        AnalysisDescriptor::BeatMarkers(_) => invalid("analysis BPM range is invalid"),
    }
}

fn timestamps(
    values: impl ExactSizeIterator<Item = (veac_ir::RationalTime, u32)>,
) -> ArtifactResult<()> {
    if values.len() > MAX_ANALYSIS_BOUNDARIES {
        return limit("analysis result exceeds its marker budget");
    }
    let mut previous = None;
    for (at, confidence) in values {
        if !at.is_valid() || at.value < 0 || confidence > CONFIDENCE_SCALE {
            return invalid("analysis marker contains invalid typed values");
        }
        if previous.is_some_and(|value| value >= at) {
            return invalid("analysis markers must be unique and sorted");
        }
        previous = Some(at);
    }
    Ok(())
}

impl SceneBoundary {
    fn values(&self) -> (veac_ir::RationalTime, u32) {
        (self.at, self.confidence_millionths)
    }
}

impl BeatMarker {
    fn values(&self) -> (veac_ir::RationalTime, u32) {
        (self.at, self.confidence_millionths)
    }
}

pub(super) fn producer(value: &ProducerFingerprint) -> ArtifactResult<()> {
    if value.name.is_empty() || value.version.is_empty() {
        return invalid("analysis producer identity is incomplete");
    }
    value.configuration.validate()
}

fn invalid<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::InvalidContract,
        message,
    ))
}

fn limit<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::ResourceLimit,
        message,
    ))
}
