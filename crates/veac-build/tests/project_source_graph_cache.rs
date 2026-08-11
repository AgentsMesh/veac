mod project_support;

use veac_build::{CancellationToken, ProjectNodeStatus};

use project_support::{single_graph, BackendMode, Fixture, TestBackend};

#[test]
fn imported_module_revision_invalidates_the_outer_project_cache() {
    let fixture = Fixture::new();
    let graph = single_graph();
    let adapter = fixture.adapter();
    let first_plan = adapter.adapt(&graph).unwrap();
    let backend = TestBackend::new(BackendMode::Good);
    let runtime = fixture.runtime(backend.clone());

    let first = runtime
        .build(&first_plan, CancellationToken::new())
        .unwrap();
    assert_eq!(first.nodes[0].status, ProjectNodeStatus::Executed);
    let cached = runtime
        .build(&first_plan, CancellationToken::new())
        .unwrap();
    assert_eq!(cached.nodes[0].status, ProjectNodeStatus::CacheHit);
    assert_eq!(backend.calls(), 1);

    std::fs::write(
        fixture.source.join("plate-module.veac"),
        "module { export fn title() -> text { \"Revised\" } }\n",
    )
    .unwrap();
    let revised_plan = adapter.adapt(&graph).unwrap();
    assert_ne!(first_plan.graph.digest(), revised_plan.graph.digest());
    let revised = runtime
        .build(&revised_plan, CancellationToken::new())
        .unwrap();
    assert_eq!(revised.nodes[0].status, ProjectNodeStatus::Executed);
    assert_eq!(backend.calls(), 2);
}
