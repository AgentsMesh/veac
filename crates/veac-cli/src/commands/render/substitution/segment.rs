use std::time::Instant;
use veac_artifact::{
    artifact_key, select_full_render_segment_while, ArtifactRecord, ArtifactStore,
    FullRenderSegmentContract, ProducerFingerprint,
};
use veac_codegen::emitter::BackendPhase;
use veac_runtime::executor::BundleExecution;

use crate::arguments::SubstitutionPolicy;
use crate::environment::Environment;
use crate::error::{CliError, CliResult};
use crate::planning::PreparedPlan;

#[cfg(test)]
#[path = "segment/store_shape_tests.rs"]
mod store_shape_tests;
#[cfg(test)]
#[path = "segment/tests.rs"]
mod tests;

pub(crate) enum SegmentDisposition {
    Disabled,
    Hit,
    Store {
        contract: FullRenderSegmentContract,
        deadline: Instant,
    },
}

pub(super) fn select(
    prepared: &mut PreparedPlan,
    store: &ArtifactStore,
    policy: SubstitutionPolicy,
    producer: Option<&ProducerFingerprint>,
    environment: &dyn Environment,
    deadline: Instant,
) -> CliResult<SegmentDisposition> {
    if policy == SubstitutionPolicy::Original {
        return Ok(SegmentDisposition::Disabled);
    }
    if Instant::now() >= deadline {
        return Err(CliError::resource_limit(
            "RENDER_SEGMENT_POSTFLIGHT_FAILED",
            "render-segment selection exceeded its wall budget",
        ));
    }
    let producer = producer.expect("non-original policy obtains an FFmpeg fingerprint");
    let contract = match FullRenderSegmentContract::new(
        &prepared.plan,
        prepared.bindings.input_substitution_proof(),
        producer.clone(),
    ) {
        Ok(value) => value,
        Err(error)
            if policy == SubstitutionPolicy::Prefer
                && error.kind != veac_artifact::ArtifactErrorKind::ResourceLimit
                && Instant::now() < deadline =>
        {
            return Ok(SegmentDisposition::Disabled);
        }
        Err(error) => return Err(segment_error(error)),
    };
    match select_full_render_segment_while(store, &contract, || Instant::now() < deadline)
        .map_err(segment_error)?
    {
        Some(artifact) => {
            if let Err(error) = environment.validate_render_segment(
                artifact.payload_path(),
                &contract,
                artifact.record(),
                deadline,
            ) {
                if policy == SubstitutionPolicy::Prefer
                    && !error.is_resource_limit()
                    && Instant::now() < deadline
                {
                    return Ok(SegmentDisposition::Disabled);
                }
                return Err(error);
            }
            prepared
                .bindings
                .bind_verified_render_segment(&prepared.plan, &contract, &artifact)
                .map_err(segment_error)?;
            Ok(SegmentDisposition::Hit)
        }
        None if policy == SubstitutionPolicy::Require => Err(CliError::new(
            "RENDER_SEGMENT_REQUIRED_MISSING",
            "no exact full-range render segment exists",
        )),
        None => Ok(SegmentDisposition::Store { contract, deadline }),
    }
}

pub(super) fn store(
    prepared: &PreparedPlan,
    store: &ArtifactStore,
    disposition: SegmentDisposition,
    execution: &BundleExecution,
    environment: &dyn Environment,
) -> CliResult {
    let SegmentDisposition::Store { contract, deadline } = disposition else {
        return Ok(());
    };
    let output = prepared
        .bindings
        .output(contract.deliverable_id())
        .ok_or_else(|| CliError::new("OUTPUT_BINDING_MISSING", "segment output is unbound"))?;
    let mut matching = execution.tasks.iter().filter(|task| {
        task.deliverable_id == *contract.deliverable_id() && task.phase == BackendPhase::Single
    });
    let task = matching
        .next()
        .ok_or_else(|| mismatch("segment render execution is missing"))?;
    if matching.next().is_some() {
        return Err(mismatch("segment render execution is ambiguous"));
    }
    let [executed] = task.outputs.as_slice() else {
        return Err(mismatch("segment render must produce exactly one output"));
    };
    let [record] = task.output_records.as_slice() else {
        return Err(mismatch("segment render output identity is unavailable"));
    };
    if executed != output {
        return Err(mismatch("segment render output path changed"));
    }
    let expected = ArtifactRecord {
        key: artifact_key(contract.descriptor()).map_err(segment_error)?,
        content: record.content.clone(),
        size_bytes: record.size_bytes,
    };
    environment.validate_render_segment(output, &contract, &expected, deadline)?;
    store
        .put_file_expected_while(
            contract.descriptor(),
            output,
            &record.content,
            record.size_bytes,
            || Instant::now() < deadline,
        )
        .map(|_| ())
        .map_err(segment_error)
}

fn segment_error(error: veac_artifact::ArtifactError) -> CliError {
    if error.kind == veac_artifact::ArtifactErrorKind::ResourceLimit {
        CliError::resource_limit("RENDER_SEGMENT_FAILED", error.to_string())
    } else {
        CliError::new("RENDER_SEGMENT_FAILED", error.to_string())
    }
}

fn mismatch(message: &str) -> CliError {
    CliError::new("RENDER_SEGMENT_OUTPUT_MISMATCH", message)
}
