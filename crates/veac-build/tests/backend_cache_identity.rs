mod project_support;
mod support;

use veac_build::*;

use project_support::{single_graph, BackendMode, Fixture, TestBackend};

#[test]
fn node_cache_misses_when_only_executor_implementation_changes() {
    let mut builder = GraphBuilder::new();
    builder.add_node(support::node("render")).unwrap();
    let graph = builder.validate().unwrap();
    let cache = MemoryBuildCache::new();
    let first = support::FakeExecutor::with_identity(b"backend-a");
    let second = support::FakeExecutor::with_identity(b"backend-b");

    let baseline = support::scheduler(1)
        .run(&graph, &first, &cache, CancellationToken::new())
        .unwrap();
    let changed = support::scheduler(1)
        .run(&graph, &second, &cache, CancellationToken::new())
        .unwrap();

    assert_eq!(baseline.nodes[0].status, NodeStatus::Executed);
    assert_eq!(changed.nodes[0].status, NodeStatus::Executed);
    assert_ne!(baseline.nodes[0].cache_key, changed.nodes[0].cache_key);
    assert_eq!(cache.len().unwrap(), 2);
}

#[test]
fn scheduler_rejects_an_invalid_executor_implementation_identity() {
    let mut builder = GraphBuilder::new();
    builder.add_node(support::node("render")).unwrap();
    let graph = builder.validate().unwrap();
    let invalid = support::FakeExecutor::with_digest(ContentDigest {
        algorithm: veac_artifact::DigestAlgorithm::Sha256,
        value: "invalid".to_owned(),
    });
    let error = support::scheduler(1)
        .run(
            &graph,
            &invalid,
            &MemoryBuildCache::new(),
            CancellationToken::new(),
        )
        .unwrap_err();
    assert_eq!(error.kind(), BuildErrorKind::InvalidContract);
    assert!(error.message().contains("implementation identity"));
}

#[test]
fn project_outer_cache_binds_the_typed_backend_identity() {
    let fixture = Fixture::new();
    let mut graph = single_graph();
    graph.instances[0].deliveries.clear();
    let plan = fixture.adapter().adapt(&graph).unwrap();
    let first = TestBackend::with_identity(BackendMode::Good, b"backend-a");
    let same = TestBackend::with_identity(BackendMode::Good, b"backend-a");
    let changed = TestBackend::with_identity(BackendMode::Good, b"backend-b");

    let baseline = fixture
        .runtime(first.clone())
        .build(&plan, CancellationToken::new())
        .unwrap();
    let cached = fixture
        .runtime(same.clone())
        .build(&plan, CancellationToken::new())
        .unwrap();
    let rebuilt = fixture
        .runtime(changed.clone())
        .build(&plan, CancellationToken::new())
        .unwrap();

    assert_eq!(baseline.nodes[0].status, ProjectNodeStatus::Executed);
    assert_eq!(cached.nodes[0].status, ProjectNodeStatus::CacheHit);
    assert_eq!(rebuilt.nodes[0].status, ProjectNodeStatus::Executed);
    assert_eq!((first.calls(), same.calls(), changed.calls()), (1, 0, 1));
    assert_ne!(baseline.nodes[0].cache_key, rebuilt.nodes[0].cache_key);
}
