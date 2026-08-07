use tempfile::tempdir;
use veac_artifact::{ArtifactRecord, ArtifactStore, ContentDigest, FullRenderSegmentContract};
use veac_codegen::emitter::BackendPhase;
use veac_runtime::executor::{BundleExecution, TaskExecution};

use super::*;
use crate::environment::Environment;
use crate::unit_tests::support::{canonical_project, FakeEnvironment, GENERATED_SOURCE};

#[test]
fn segment_store_rejects_every_malformed_execution_shape() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let environment = FakeEnvironment::success();
    let mut prepared =
        crate::planning::prepare_with_material_root(&project, None, None, &environment).unwrap();
    let producer = super::super::producer(environment.ffmpeg_fingerprint().unwrap()).unwrap();
    let contract = FullRenderSegmentContract::new(
        &prepared.plan,
        prepared.bindings.input_substitution_proof(),
        producer,
    )
    .unwrap();
    let artifact_store = ArtifactStore::new(temp.path().join("store"));

    assert!(
        attempt(&prepared, &artifact_store, &contract, vec![], &environment)
            .contains("OUTPUT_BINDING_MISSING")
    );
    crate::output::bind_render_outputs(&mut prepared, None).unwrap();
    assert!(
        attempt(&prepared, &artifact_store, &contract, vec![], &environment)
            .contains("execution is missing")
    );

    let output = prepared
        .bindings
        .output(contract.deliverable_id())
        .unwrap()
        .to_owned();
    let base = TaskExecution {
        deliverable_id: contract.deliverable_id().clone(),
        phase: BackendPhase::Single,
        cache_hit: false,
        outputs: vec![],
        output_records: vec![],
        checkpoint: record(ContentDigest::sha256(b"checkpoint"), 10),
    };
    assert!(attempt(
        &prepared,
        &artifact_store,
        &contract,
        vec![base.clone(), base.clone()],
        &environment,
    )
    .contains("execution is ambiguous"));
    assert!(attempt(
        &prepared,
        &artifact_store,
        &contract,
        vec![base.clone()],
        &environment,
    )
    .contains("exactly one output"));
    let mut no_record = base.clone();
    no_record.outputs = vec![output];
    assert!(attempt(
        &prepared,
        &artifact_store,
        &contract,
        vec![no_record],
        &environment,
    )
    .contains("identity is unavailable"));
    let mut changed = base;
    changed.outputs = vec![temp.path().join("changed.mp4")];
    changed.output_records = vec![record(ContentDigest::sha256(b"rendered"), 8)];
    assert!(attempt(
        &prepared,
        &artifact_store,
        &contract,
        vec![changed],
        &environment,
    )
    .contains("output path changed"));
}

fn attempt(
    prepared: &crate::planning::PreparedPlan,
    store_ref: &ArtifactStore,
    contract: &FullRenderSegmentContract,
    tasks: Vec<TaskExecution>,
    environment: &dyn Environment,
) -> String {
    store(
        prepared,
        store_ref,
        SegmentDisposition::Store {
            contract: Box::new(contract.clone()),
            deadline: Instant::now() + std::time::Duration::from_secs(2),
        },
        &BundleExecution { tasks },
        environment,
    )
    .unwrap_err()
    .to_string()
}

fn record(content: ContentDigest, size_bytes: u64) -> ArtifactRecord {
    ArtifactRecord {
        key: ContentDigest::sha256(b"fixture-key"),
        content,
        size_bytes,
    }
}
