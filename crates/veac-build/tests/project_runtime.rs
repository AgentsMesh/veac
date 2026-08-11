mod project_support;

use veac_build::*;
use veac_project::{DeliveryKind, MediaType, OutputId, ProjectOutput};

use project_support::{graph, BackendMode, Fixture, TestBackend};

#[test]
fn project_runtime_publishes_cas_artifacts_deliveries_and_cache_hits() {
    let fixture = Fixture::new();
    let plan = fixture.adapter().adapt(&graph()).unwrap();
    let backend = TestBackend::new(BackendMode::Good);
    let runtime = fixture.runtime(backend.clone());

    let first = runtime.build(&plan, CancellationToken::new()).unwrap();
    assert_eq!(first.outcome, ProjectBuildOutcome::Succeeded);
    assert_eq!(backend.calls(), 3);
    assert!(first
        .nodes
        .iter()
        .all(|node| node.status == ProjectNodeStatus::Executed));
    assert!(first
        .deliveries
        .iter()
        .all(|delivery| delivery.status == DeliveryStatus::Published));
    for delivery in &first.deliveries {
        let artifact = delivery.artifact.as_ref().unwrap();
        assert!(runtime.artifact_store().open(artifact).unwrap().is_some());
        assert!(fixture
            .temp
            .path()
            .join("delivery")
            .join(&delivery.destination)
            .is_file());
    }

    let second = runtime.build(&plan, CancellationToken::new()).unwrap();
    assert_eq!(second.outcome, ProjectBuildOutcome::Succeeded);
    assert_eq!(backend.calls(), 3);
    assert!(second
        .nodes
        .iter()
        .all(|node| node.status == ProjectNodeStatus::CacheHit));
    assert_eq!(
        first
            .nodes
            .iter()
            .flat_map(|node| &node.outputs)
            .collect::<Vec<_>>(),
        second
            .nodes
            .iter()
            .flat_map(|node| &node.outputs)
            .collect::<Vec<_>>()
    );
}

#[test]
fn project_receipt_is_versioned_round_trippable_and_schema_visible() {
    let fixture = Fixture::new();
    let plan = fixture.adapter().adapt(&graph()).unwrap();
    let receipt = fixture
        .runtime(TestBackend::new(BackendMode::Good))
        .build(&plan, CancellationToken::new())
        .unwrap();

    assert_eq!(receipt.version, PROJECT_BUILD_RECEIPT_VERSION);
    let canonical = canonical_project_receipt_bytes(&receipt).unwrap();
    let decoded: ProjectBuildReceipt = serde_json::from_slice(&canonical).unwrap();
    assert_eq!(decoded, receipt);
    let mut value = serde_json::to_value(&receipt).unwrap();
    value["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<ProjectBuildReceipt>(value).is_err());

    let schema = project_build_receipt_json_schema().unwrap();
    assert_eq!(schema["title"], "ProjectBuildReceipt");
    let encoded = serde_json::to_string(&schema).unwrap();
    for expected in [
        "ProjectBuildOutcome",
        "ProjectNodeProvenance",
        "ProjectDeliveryReceipt",
        "delivery_failed",
        "cache_hit",
    ] {
        assert!(encoded.contains(expected), "schema omitted {expected}");
    }
}

#[test]
fn backend_errors_have_a_closed_failure_classification() {
    let failed = ProjectBackendError::failed("failed");
    let cancelled = ProjectBackendError::cancelled("cancelled");
    assert_eq!(failed.kind(), ProjectBackendErrorKind::Failed);
    assert_eq!(failed.message(), "failed");
    assert_eq!(failed.to_string(), "failed");
    assert_eq!(cancelled.kind(), ProjectBackendErrorKind::Cancelled);
    assert_eq!(cancelled.message(), "cancelled");
}

#[test]
fn project_output_kinds_receive_distinct_cas_descriptors() {
    let fixture = Fixture::new();
    let mut value = project_support::single_graph();
    value.instances[0].outputs = vec![
        ProjectOutput::Media {
            id: OutputId::from("video"),
            media_type: MediaType::Video,
        },
        ProjectOutput::Media {
            id: OutputId::from("audio"),
            media_type: MediaType::Audio,
        },
        ProjectOutput::Media {
            id: OutputId::from("image"),
            media_type: MediaType::Image,
        },
        ProjectOutput::Data {
            id: OutputId::from("report"),
            schema: Some("veac.test-report".to_owned()),
        },
        ProjectOutput::Directory {
            id: OutputId::from("package"),
        },
    ];
    value.instances[0].deliveries.clear();
    let plan = fixture.adapter().adapt(&value).unwrap();
    let runtime = fixture.runtime(TestBackend::new(BackendMode::Good));
    let receipt = runtime.build(&plan, CancellationToken::new()).unwrap();

    let kinds = receipt.nodes[0]
        .outputs
        .iter()
        .map(|output| {
            let artifact = runtime
                .artifact_store()
                .open(&output.artifact)
                .unwrap()
                .unwrap();
            (output.output.as_str(), artifact.descriptor().kind())
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(kinds["video"], veac_artifact::ArtifactKind::VideoMaster);
    assert_eq!(kinds["audio"], veac_artifact::ArtifactKind::AudioFile);
    assert_eq!(kinds["image"], veac_artifact::ArtifactKind::StillImage);
    assert_eq!(kinds["report"], veac_artifact::ArtifactKind::CaptionSidecar);
    assert_eq!(
        kinds["package"],
        veac_artifact::ArtifactKind::AdaptivePackage
    );
}

#[test]
fn directory_outputs_are_archived_in_cas_and_delivered_idempotently() {
    let fixture = Fixture::new();
    let mut value = project_support::single_graph();
    value.instances[0].outputs = vec![ProjectOutput::Directory {
        id: OutputId::from("bundle"),
    }];
    value.instances[0].deliveries[0].output = OutputId::from("bundle");
    value.instances[0].deliveries[0].kind = DeliveryKind::Directory;
    value.instances[0].deliveries[0].destination =
        veac_project::ProjectPath::new("evidence/bundle");
    let plan = fixture.adapter().adapt(&value).unwrap();
    let backend = TestBackend::new(BackendMode::Good);
    let runtime = fixture.runtime(backend.clone());

    let first = runtime.build(&plan, CancellationToken::new()).unwrap();
    assert_eq!(first.outcome, ProjectBuildOutcome::Succeeded);
    let delivered = fixture
        .temp
        .path()
        .join("delivery/evidence/bundle/result.txt");
    assert!(delivered.is_file());
    let second = runtime.build(&plan, CancellationToken::new()).unwrap();
    assert_eq!(second.outcome, ProjectBuildOutcome::Succeeded);
    assert_eq!(backend.calls(), 1);
    assert_eq!(second.nodes[0].status, ProjectNodeStatus::CacheHit);
}
