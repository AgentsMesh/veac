use std::time::Instant;
use veac_artifact::{ProducerFingerprint, ProxySelectionRequest};
use veac_ir::MaterialKind;
use veac_plan::ResolvedInputKind;

use crate::arguments::SubstitutionPolicy;
use crate::error::{CliError, CliResult};
use crate::planning::PreparedPlan;

mod clock;
mod descriptor;
mod profile;
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
    let profile = profile::resolve(&prepared.plan.output)?;
    if profile.is_empty() {
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
        let needs_video = input.video.is_some() && profile.raster.is_some();
        let needs_audio = input.audio.is_some() && profile.audio.is_some();
        if !needs_video && !needs_audio {
            continue;
        }
        let source_identity = descriptor::source_identity(&input.observed_identity)?;
        let video = input
            .video
            .as_ref()
            .zip(profile.raster.as_ref())
            .map(|(stream, raster)| {
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
                descriptor::video(
                    stream.selection,
                    source_clock,
                    &source_identity,
                    producer,
                    raster,
                )
            })
            .transpose()?;
        let audio = profile.audio.as_ref().and_then(|settings| {
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
                descriptor::audio(
                    stream.selection,
                    source_clock,
                    settings.sample_rate,
                    settings.channels,
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

fn artifact_error(error: veac_artifact::ArtifactError) -> CliError {
    CliError::new("PROXY_SELECTION_FAILED", error.to_string())
}
