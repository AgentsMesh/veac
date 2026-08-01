use veac_artifact::{
    ContentDigest, DigestAlgorithm, MediaArtifactRequest, MediaArtifactSpec, ProducerFingerprint,
    ProxyAudioSpec, ProxyVideoSpec, SourceClockSpec,
};
use veac_ir::{HashAlgorithm, RasterSettings, StreamSelection};

use crate::error::{CliError, CliResult};

pub(super) fn video(
    source_stream: StreamSelection,
    source_clock: SourceClockSpec,
    source: &ContentDigest,
    producer: &ProducerFingerprint,
    raster: &RasterSettings,
) -> CliResult<veac_artifact::ArtifactDescriptor> {
    descriptor(
        source,
        producer,
        MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
            source_stream,
            source_clock,
            width: raster.width,
            height: raster.height,
            frame_rate: raster.frame_rate,
            crf: 28,
        }),
    )
}

pub(super) fn audio(
    source_stream: StreamSelection,
    source_clock: SourceClockSpec,
    sample_rate: u32,
    channels: u8,
    source: &ContentDigest,
    producer: &ProducerFingerprint,
) -> CliResult<veac_artifact::ArtifactDescriptor> {
    descriptor(
        source,
        producer,
        MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream,
            source_clock,
            sample_rate,
            channels,
        }),
    )
}

fn descriptor(
    source: &ContentDigest,
    producer: &ProducerFingerprint,
    spec: MediaArtifactSpec,
) -> CliResult<veac_artifact::ArtifactDescriptor> {
    MediaArtifactRequest {
        source_identity: source.clone(),
        producer: producer.clone(),
        spec,
    }
    .descriptor()
    .map_err(super::artifact_error)
}

pub(super) fn source_identity(identity: &veac_ir::MediaIdentity) -> CliResult<ContentDigest> {
    if identity.algorithm != HashAlgorithm::Sha256 {
        return Err(CliError::new(
            "PROXY_IDENTITY_UNSUPPORTED",
            "automatic proxy selection requires SHA-256 input identity",
        ));
    }
    Ok(ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: identity.digest.clone(),
    })
}
