mod proxy;
mod segment;

use std::time::{Duration, Instant};
use veac_artifact::{ArtifactStore, ProducerFingerprint};
use veac_runtime::executor::FfmpegFingerprint;

use crate::arguments::SubstitutionPolicy;
use crate::error::CliResult;
use crate::planning::PreparedPlan;

pub(super) use segment::SegmentDisposition;

pub(super) fn prepare(
    prepared: &mut PreparedPlan,
    store: &ArtifactStore,
    proxy_policy: SubstitutionPolicy,
    segment_policy: SubstitutionPolicy,
    fingerprint: Option<FfmpegFingerprint>,
    environment: &dyn crate::environment::Environment,
) -> CliResult<SegmentDisposition> {
    let deadline =
        Instant::now() + Duration::from_secs(veac_artifact::MAX_MEDIA_DERIVATION_WALL_SECONDS);
    let producer = fingerprint.map(producer).transpose()?;
    proxy::apply(
        prepared,
        store,
        proxy_policy,
        producer.as_ref(),
        environment,
        deadline,
    )?;
    segment::select(
        prepared,
        store,
        segment_policy,
        producer.as_ref(),
        environment,
        deadline,
    )
}

pub(super) fn store(
    prepared: &PreparedPlan,
    artifact_store: &ArtifactStore,
    disposition: SegmentDisposition,
    execution: &veac_runtime::executor::BundleExecution,
    environment: &dyn crate::environment::Environment,
) -> CliResult {
    segment::store(
        prepared,
        artifact_store,
        disposition,
        execution,
        environment,
    )
}

fn producer(fingerprint: FfmpegFingerprint) -> CliResult<ProducerFingerprint> {
    veac_runtime::workflow::media_artifact_producer(&fingerprint)
        .map_err(|error| crate::CliError::new("FFMPEG_FINGERPRINT_INVALID", error.to_string()))
}
