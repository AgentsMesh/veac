use veac_ir::{HashAlgorithm, MediaIdentity, RationalTime, StreamSelection, TimeRange};
use veac_plan::ResolvedInput;

use super::{BoundAudioFacts, BoundVideoFacts, MediaRole, SourceClock};
use crate::{
    ArtifactDependencyRole, ArtifactError, ArtifactErrorKind, ArtifactKind, ArtifactParameters,
    ArtifactResult, ContentDigest, SourceClockSpec, VerifiedArtifact,
};

pub(super) fn binding(
    input: &ResolvedInput,
    role: MediaRole,
    artifact: &VerifiedArtifact,
) -> ArtifactResult<(
    StreamSelection,
    SourceClock,
    Option<BoundVideoFacts>,
    Option<BoundAudioFacts>,
)> {
    let expected_kind = match role {
        MediaRole::Video => ArtifactKind::ProxyVideo,
        MediaRole::Audio => ArtifactKind::ProxyAudio,
    };
    let source = media_digest(&input.observed_identity)?;
    let dependencies: Vec<_> = artifact
        .descriptor()
        .dependencies
        .iter()
        .filter(|dependency| dependency.role == ArtifactDependencyRole::Input)
        .collect();
    if artifact.descriptor().kind() != expected_kind
        || dependencies.len() != 1
        || dependencies[0].identity != source
    {
        return invalid("verified proxy is not bound to the resolved input and media role");
    }
    let (source_stream, clock, video_facts, audio_facts) =
        match (role, &artifact.descriptor().parameters) {
            (MediaRole::Video, ArtifactParameters::ProxyVideo(value)) => (
                value.source_stream,
                value.source_clock,
                Some(BoundVideoFacts::proxy(value)),
                None,
            ),
            (MediaRole::Audio, ArtifactParameters::ProxyAudio(value)) => (
                value.source_stream,
                value.source_clock,
                None,
                Some(BoundAudioFacts::proxy(value)),
            ),
            _ => return invalid("verified proxy parameters do not match its media role"),
        };
    if source_stream != selected(input, role)? {
        return invalid("verified proxy source stream differs from the resolved input stream");
    }
    Ok((
        StreamSelection {
            global_index: 0,
            type_index: 0,
        },
        source_clock(clock)?,
        video_facts,
        audio_facts,
    ))
}

fn selected(input: &ResolvedInput, role: MediaRole) -> ArtifactResult<StreamSelection> {
    match role {
        MediaRole::Video => input.video.as_ref().map(|value| value.selection),
        MediaRole::Audio => input.audio.as_ref().map(|value| value.selection),
    }
    .ok_or_else(|| ArtifactError::new(ArtifactErrorKind::InvalidContract, "proxy role is absent"))
}

fn source_clock(spec: SourceClockSpec) -> ArtifactResult<SourceClock> {
    spec.validate()?;
    match spec {
        SourceClockSpec::Identity { duration } => SourceClock::bounded(
            TimeRange::new(
                RationalTime::zero(duration.timescale).map_err(time_error)?,
                duration,
            )
            .map_err(time_error)?,
            RationalTime::zero(duration.timescale).map_err(time_error)?,
        ),
        SourceClockSpec::Bounded { logical_range } => SourceClock::bounded(
            logical_range,
            RationalTime::zero(logical_range.start.timescale).map_err(time_error)?,
        ),
    }
}

fn time_error(error: veac_ir::TimeError) -> ArtifactError {
    ArtifactError::with_source(
        ArtifactErrorKind::InvalidContract,
        "proxy source clock is invalid",
        error,
    )
}

pub(super) fn media_digest(identity: &MediaIdentity) -> ArtifactResult<ContentDigest> {
    if identity.algorithm != HashAlgorithm::Sha256 {
        return invalid("proxy substitution requires a SHA-256 source identity");
    }
    let digest = ContentDigest {
        algorithm: crate::DigestAlgorithm::Sha256,
        value: identity.digest.clone(),
    };
    digest.validate()?;
    Ok(digest)
}

fn invalid<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::InvalidContract,
        message,
    ))
}

#[cfg(test)]
#[path = "proxy/tests.rs"]
mod tests;
