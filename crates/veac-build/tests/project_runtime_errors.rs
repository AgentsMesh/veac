mod project_support;

use veac_build::*;
use veac_project::{DeliveryKind, OutputId, ProjectOutput};

use project_support::{graph, single_graph, BackendMode, Fixture, TestBackend};

#[test]
fn backend_and_output_contract_failures_stop_the_project_dag() {
    for (mode, fragment) in [
        (BackendMode::Fail, "backend failed"),
        (BackendMode::Missing, "output count"),
        (BackendMode::Duplicate, "duplicate output"),
        (BackendMode::Escape, "canonical and relative"),
        (BackendMode::WrongId, "omitted output"),
        (BackendMode::Directory, "regular non-symlink file"),
    ] {
        let fixture = Fixture::new();
        let plan = fixture.adapter().adapt(&single_graph()).unwrap();
        let receipt = fixture
            .runtime(TestBackend::new(mode))
            .build(&plan, CancellationToken::new())
            .unwrap();
        assert_eq!(receipt.outcome, ProjectBuildOutcome::BuildFailed);
        assert_eq!(receipt.nodes[0].status, ProjectNodeStatus::Failed);
        assert!(receipt.nodes[0]
            .message
            .as_deref()
            .unwrap()
            .contains(fragment));
        assert!(receipt.deliveries.is_empty());
    }
}

#[cfg(unix)]
#[test]
fn backend_outputs_cannot_follow_symlinks_outside_the_workspace_contract() {
    let fixture = Fixture::new();
    let plan = fixture.adapter().adapt(&single_graph()).unwrap();
    let receipt = fixture
        .runtime(TestBackend::new(BackendMode::Symlink))
        .build(&plan, CancellationToken::new())
        .unwrap();

    assert_eq!(receipt.outcome, ProjectBuildOutcome::BuildFailed);
    assert!(receipt.nodes[0]
        .message
        .as_deref()
        .unwrap()
        .contains("symlink"));
}

#[test]
fn cancellation_is_preserved_as_a_distinct_project_outcome() {
    let fixture = Fixture::new();
    let plan = fixture.adapter().adapt(&single_graph()).unwrap();
    let token = CancellationToken::new();
    token.cancel();
    let receipt = fixture
        .runtime(TestBackend::new(BackendMode::Good))
        .build(&plan, token)
        .unwrap();
    assert_eq!(receipt.outcome, ProjectBuildOutcome::Cancelled);
    assert_eq!(receipt.nodes[0].status, ProjectNodeStatus::Cancelled);

    let fixture = Fixture::new();
    let plan = fixture.adapter().adapt(&single_graph()).unwrap();
    let receipt = fixture
        .runtime(TestBackend::new(BackendMode::Cancel))
        .build(&plan, CancellationToken::new())
        .unwrap();
    assert_eq!(receipt.outcome, ProjectBuildOutcome::Cancelled);

    let fixture = Fixture::new();
    let plan = fixture.adapter().adapt(&single_graph()).unwrap();
    let receipt = fixture
        .runtime(TestBackend::new(BackendMode::Cancelled))
        .build(&plan, CancellationToken::new())
        .unwrap();
    assert_eq!(receipt.outcome, ProjectBuildOutcome::Cancelled);
}

#[test]
fn project_receipt_distinguishes_failed_and_blocked_nodes() {
    let fixture = Fixture::new();
    let plan = fixture.adapter().adapt(&graph()).unwrap();
    let receipt = fixture
        .runtime(TestBackend::new(BackendMode::Fail))
        .build(&plan, CancellationToken::new())
        .unwrap();
    assert_eq!(receipt.outcome, ProjectBuildOutcome::BuildFailed);
    assert!(receipt
        .nodes
        .iter()
        .any(|node| node.status == ProjectNodeStatus::Failed));
    assert!(receipt
        .nodes
        .iter()
        .any(|node| node.status == ProjectNodeStatus::Blocked));
}

#[test]
fn delivery_failure_is_atomic_and_skips_later_publications() {
    let fixture = Fixture::new();
    let mut value = graph();
    for instance in &mut value.instances {
        instance.deliveries[0].destination =
            veac_project::ProjectPath::new(format!("blocked/{}.bin", instance.id));
    }
    let plan = fixture.adapter().adapt(&value).unwrap();
    let runtime = fixture.runtime(TestBackend::new(BackendMode::Good));
    std::fs::write(
        fixture.temp.path().join("delivery/blocked"),
        b"not a directory",
    )
    .unwrap();

    let receipt = runtime.build(&plan, CancellationToken::new()).unwrap();
    assert_eq!(receipt.outcome, ProjectBuildOutcome::DeliveryFailed);
    assert_eq!(receipt.deliveries[0].status, DeliveryStatus::Failed);
    assert!(receipt.deliveries[1..]
        .iter()
        .all(|delivery| delivery.status == DeliveryStatus::Skipped));
    assert!(receipt.deliveries[0]
        .message
        .as_deref()
        .unwrap()
        .contains("non-directory"));
}

#[test]
fn delivery_referencing_a_missing_output_fails_closed() {
    let fixture = Fixture::new();
    let mut plan = fixture.adapter().adapt(&single_graph()).unwrap();
    plan.deliveries[0].delivery.output = OutputId::from("missing");
    let receipt = fixture
        .runtime(TestBackend::new(BackendMode::Good))
        .build(&plan, CancellationToken::new())
        .unwrap();
    assert_eq!(receipt.outcome, ProjectBuildOutcome::DeliveryFailed);
    assert_eq!(receipt.deliveries[0].status, DeliveryStatus::Failed);
    assert!(receipt.deliveries[0]
        .message
        .as_deref()
        .unwrap()
        .contains("missing project output"));
}

#[test]
fn runtime_roots_reject_non_directory_authorities() {
    let fixture = Fixture::new();
    let file = fixture.temp.path().join("file-root");
    std::fs::write(&file, b"file").unwrap();
    let result = ProjectBuildRuntime::new(
        BuildLimits::new(1, ResourceClaim::new(1, 1, 0)).unwrap(),
        veac_artifact::ArtifactStore::new(fixture.temp.path().join("artifacts")),
        &file,
        fixture.temp.path().join("staging"),
        fixture.temp.path().join("delivery"),
        TestBackend::new(BackendMode::Good),
    );
    assert!(result.err().unwrap().message().contains("lease"));
}

#[test]
fn changed_existing_directory_delivery_fails_without_overwriting_it() {
    let fixture = Fixture::new();
    let mut value = single_graph();
    value.instances[0].outputs = vec![ProjectOutput::Directory {
        id: OutputId::from("bundle"),
    }];
    value.instances[0].deliveries[0].output = OutputId::from("bundle");
    value.instances[0].deliveries[0].kind = DeliveryKind::Directory;
    value.instances[0].deliveries[0].destination = veac_project::ProjectPath::new("bundle");
    let plan = fixture.adapter().adapt(&value).unwrap();
    let runtime = fixture.runtime(TestBackend::new(BackendMode::Good));
    assert_eq!(
        runtime
            .build(&plan, CancellationToken::new())
            .unwrap()
            .outcome,
        ProjectBuildOutcome::Succeeded
    );
    let delivered = fixture.temp.path().join("delivery/bundle/result.txt");
    std::fs::write(&delivered, b"changed").unwrap();
    let receipt = runtime.build(&plan, CancellationToken::new()).unwrap();
    assert_eq!(receipt.outcome, ProjectBuildOutcome::DeliveryFailed);
    assert_eq!(std::fs::read(delivered).unwrap(), b"changed");
}
