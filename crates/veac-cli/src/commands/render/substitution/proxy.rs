use std::time::Instant;
use veac_artifact::{
    ContentDigest, DigestAlgorithm, MediaArtifactRequest, MediaArtifactSpec, ProducerFingerprint,
    ProxyAudioSpec, ProxySelectionRequest, ProxyVideoSpec, SourceClockSpec,
};
use veac_ir::{HashAlgorithm, MaterialKind, StreamSelection};
use veac_plan::ResolvedInputKind;

use crate::arguments::SubstitutionPolicy;
use crate::error::{CliError, CliResult};
use crate::planning::PreparedPlan;

mod clock;
mod verify;

#[cfg(test)]
#[path = "proxy/deadline_tests.rs"]
mod deadline_tests;

pub(super) fn apply(
    prepared: &mut PreparedPlan,
    store: &veac_artifact::ArtifactStore,
    policy: SubstitutionPolicy,
    producer: Option<&ProducerFingerprint>,
    environment: &dyn crate::environment::Environment,
    deadline: Instant,
) -> CliResult {
    if policy == SubstitutionPolicy::Original {
        return Ok(());
    }
    if Instant::now() >= deadline {
        return Err(CliError::resource_limit(
            "PROXY_POSTFLIGHT_FAILED",
            "proxy selection exceeded its wall budget",
        ));
    }
    let producer = producer.expect("non-original policy obtains an FFmpeg fingerprint");
    for input in &prepared.plan.inputs {
        if matches!(
            input.kind,
            ResolvedInputKind::Media {
                material_kind: MaterialKind::Image
            }
        ) {
            continue;
        }
        let source_identity = source_identity(&input.observed_identity)?;
        let video = input
            .video
            .as_ref()
            .map(|stream| {
                let source_clock = clock::resolve(
                    &input.id,
                    prepared.plan.header.source.timebase,
                    input
                        .probe
                        .as_ref()
                        .and_then(|value| value.container_duration),
                    stream.start_time,
                    stream.duration,
                    "video",
                )?;
                video_descriptor(
                    prepared,
                    stream.selection,
                    source_clock,
                    &source_identity,
                    producer,
                )
            })
            .transpose()?;
        let audio = audio_settings(prepared).and_then(|settings| {
            input.audio.as_ref().map(|stream| {
                let source_clock = clock::resolve(
                    &input.id,
                    prepared.plan.header.source.timebase,
                    input
                        .probe
                        .as_ref()
                        .and_then(|value| value.container_duration),
                    stream.start_time,
                    stream.duration,
                    "audio",
                )?;
                audio_descriptor(
                    stream.selection,
                    source_clock,
                    settings,
                    &source_identity,
                    producer,
                )
            })
        });
        let audio = audio.transpose()?;
        if video.is_none() && audio.is_none() {
            continue;
        }
        let request = ProxySelectionRequest {
            source_identity,
            video,
            audio,
        };
        let selection = verify::select(store, &request, policy, environment, deadline)?;
        let missing = request.video.is_some() && selection.video.is_none()
            || request.audio.is_some() && selection.audio.is_none();
        if missing && policy == SubstitutionPolicy::Require {
            return Err(CliError::new(
                "PROXY_REQUIRED_MISSING",
                format!("input {} has no exact verified proxy", input.id),
            ));
        }
        prepared
            .bindings
            .bind_proxy_selection(input, &selection)
            .map_err(artifact_error)?;
    }
    Ok(())
}

fn video_descriptor(
    prepared: &PreparedPlan,
    source_stream: StreamSelection,
    source_clock: SourceClockSpec,
    source: &ContentDigest,
    producer: &ProducerFingerprint,
) -> CliResult<veac_artifact::ArtifactDescriptor> {
    descriptor(
        source,
        producer,
        MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
            source_stream,
            source_clock,
            width: prepared.plan.output.width,
            height: prepared.plan.output.height,
            frame_rate: prepared.plan.output.frame_rate,
            crf: 28,
        }),
    )
}

fn audio_descriptor(
    source_stream: StreamSelection,
    source_clock: SourceClockSpec,
    settings: &veac_ir::AudioOutput,
    source: &ContentDigest,
    producer: &ProducerFingerprint,
) -> CliResult<veac_artifact::ArtifactDescriptor> {
    descriptor(
        source,
        producer,
        MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream,
            source_clock,
            sample_rate: settings.sample_rate,
            channels: settings.channels,
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
    .map_err(artifact_error)
}

fn source_identity(identity: &veac_ir::MediaIdentity) -> CliResult<ContentDigest> {
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

fn audio_settings(prepared: &PreparedPlan) -> Option<&veac_ir::AudioOutput> {
    prepared
        .plan
        .output
        .video_deliverable()
        .and_then(|(_, value)| value.audio.as_ref())
}

fn artifact_error(error: veac_artifact::ArtifactError) -> CliError {
    CliError::new("PROXY_SELECTION_FAILED", error.to_string())
}
