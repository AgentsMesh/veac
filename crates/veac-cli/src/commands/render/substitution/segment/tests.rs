use tempfile::tempdir;
use veac_artifact::{artifact_key, ArtifactRecord, ArtifactStore, ContentDigest};
use veac_codegen::emitter::BackendPhase;
use veac_runtime::executor::{BundleExecution, TaskExecution};

use super::*;
use crate::environment::Environment;
use crate::unit_tests::support::{canonical_project, FakeEnvironment, GENERATED_SOURCE};

#[test]
fn invalid_ffmpeg_fingerprints_remain_typed_cli_errors() {
    let fingerprint = veac_runtime::executor::FfmpegFingerprint {
        version: " ".into(),
        configuration: ContentDigest::sha256(b"configuration"),
    };

    let error = super::super::producer(fingerprint).unwrap_err();

    assert!(error.to_string().contains("FFMPEG_FINGERPRINT_INVALID"));
}

#[test]
fn post_render_output_replacement_cannot_be_promoted_as_an_exact_segment() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let environment = FakeEnvironment::success();
    let mut prepared =
        crate::planning::prepare_with_material_root(&project, None, None, &environment).unwrap();
    crate::output::bind_render_outputs(&mut prepared, None).unwrap();
    let producer = super::super::producer(environment.ffmpeg_fingerprint().unwrap()).unwrap();
    let contract = FullRenderSegmentContract::new(
        &prepared.plan,
        prepared.bindings.input_substitution_proof(),
        producer,
    )
    .unwrap();
    let descriptor = contract.descriptor().clone();
    let output = prepared
        .bindings
        .output(contract.deliverable_id())
        .unwrap()
        .to_path_buf();
    std::fs::write(&output, b"replaced").unwrap();
    let rendered = ContentDigest::sha256(b"rendered");
    let execution = BundleExecution {
        tasks: vec![TaskExecution {
            deliverable_id: contract.deliverable_id().clone(),
            phase: BackendPhase::Single,
            cache_hit: false,
            outputs: vec![output],
            output_records: vec![record(rendered, 8)],
            checkpoint: record(ContentDigest::sha256(b"checkpoint"), 10),
        }],
    };
    let artifact_store = ArtifactStore::new(temp.path().join(".veac-artifacts"));

    let error = store(
        &prepared,
        &artifact_store,
        SegmentDisposition::Store {
            contract: Box::new(contract),
            deadline: Instant::now() + std::time::Duration::from_secs(1),
        },
        &execution,
        &environment,
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("RENDER_SEGMENT_POSTFLIGHT_FAILED"));
    assert!(artifact_store
        .get(&artifact_key(&descriptor).unwrap())
        .unwrap()
        .is_none());
}

#[test]
fn expired_prefer_selection_propagates_without_starting_ffmpeg() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let environment = FakeEnvironment::success();
    let mut prepared =
        crate::planning::prepare_with_material_root(&project, None, None, &environment).unwrap();
    let producer = super::super::producer(environment.ffmpeg_fingerprint().unwrap()).unwrap();
    let error = select(
        &mut prepared,
        &ArtifactStore::new(temp.path().join("store")),
        SubstitutionPolicy::Prefer,
        Some(&producer),
        &environment,
        Instant::now(),
    )
    .err()
    .expect("expired selection must fail");
    assert!(error.is_resource_limit());
    assert!(environment.executed.borrow().is_empty());
}

#[test]
fn prefer_propagates_a_contract_resource_limit_without_starting_ffmpeg() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let environment = FakeEnvironment::success();
    let mut prepared =
        crate::planning::prepare_with_material_root(&project, None, None, &environment).unwrap();
    let mut producer = super::super::producer(environment.ffmpeg_fingerprint().unwrap()).unwrap();
    producer.version = "v".repeat(veac_artifact::MAX_ARTIFACT_JSON_STRING_BYTES);
    let error = select(
        &mut prepared,
        &ArtifactStore::new(temp.path().join("store")),
        SubstitutionPolicy::Prefer,
        Some(&producer),
        &environment,
        Instant::now() + std::time::Duration::from_secs(1),
    )
    .err()
    .expect("a contract resource limit must not fall back");
    assert!(error.is_resource_limit());
    assert!(environment.executed.borrow().is_empty());
}

#[test]
fn prefer_does_not_select_a_segment_from_another_producer_contract() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let environment = FakeEnvironment::success();
    let mut prepared =
        crate::planning::prepare_with_material_root(&project, None, None, &environment).unwrap();
    let producer = super::super::producer(environment.ffmpeg_fingerprint().unwrap()).unwrap();
    let mut previous = producer.clone();
    previous.configuration = ContentDigest::sha256(b"previous render implementation");
    let old_contract = FullRenderSegmentContract::new(
        &prepared.plan,
        prepared.bindings.input_substitution_proof(),
        previous,
    )
    .unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    store
        .put(old_contract.descriptor(), b"old segment")
        .unwrap();

    let disposition = select(
        &mut prepared,
        &store,
        SubstitutionPolicy::Prefer,
        Some(&producer),
        &environment,
        Instant::now() + std::time::Duration::from_secs(1),
    )
    .unwrap();
    assert!(matches!(disposition, SegmentDisposition::Store { .. }));
    assert!(environment.executed.borrow().is_empty());
}

fn record(content: ContentDigest, size_bytes: u64) -> ArtifactRecord {
    ArtifactRecord {
        key: ContentDigest::sha256(b"fixture-key"),
        content,
        size_bytes,
    }
}
