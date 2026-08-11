mod project_support;

use veac_build::{
    BuildAction, CancellationToken, ProjectAction, ProjectBuildPlan, ProjectNodeStatus,
};
use veac_project::{ProfileId, TargetInstanceId};

use project_support::{graph, BackendMode, Fixture, TestBackend, REVISED_EVIDENCE_HELPER};

#[test]
fn imported_evidence_module_changes_action_and_graph_digests() {
    let fixture = Fixture::new();
    let graph = graph();
    let adapter = fixture.adapter();
    let baseline = adapter.adapt(&graph).unwrap();
    let baseline_action = evidence_action(&baseline);
    let (baseline_contract, baseline_revision) = evidence_source(baseline_action);

    revise_evidence_helper(&fixture);
    let revised = adapter.adapt(&graph).unwrap();
    let revised_action = evidence_action(&revised);
    let (revised_contract, revised_revision) = evidence_source(revised_action);

    assert_eq!(baseline_contract, revised_contract);
    assert_ne!(baseline_revision, revised_revision);
    assert_ne!(
        baseline_action.canonical_bytes().unwrap(),
        revised_action.canonical_bytes().unwrap()
    );
    assert_ne!(baseline.graph.digest(), revised.graph.digest());
}

#[test]
fn imported_evidence_change_misses_only_the_evidence_cache_entry() {
    let fixture = Fixture::new();
    let graph = graph();
    let adapter = fixture.adapter();
    let baseline = adapter.adapt(&graph).unwrap();
    let backend = TestBackend::new(BackendMode::Good);
    let runtime = fixture.runtime(backend.clone());

    let first = runtime.build(&baseline, CancellationToken::new()).unwrap();
    assert!(first
        .nodes
        .iter()
        .all(|node| node.status == ProjectNodeStatus::Executed));
    let cached = runtime.build(&baseline, CancellationToken::new()).unwrap();
    assert!(cached
        .nodes
        .iter()
        .all(|node| node.status == ProjectNodeStatus::CacheHit));

    revise_evidence_helper(&fixture);
    let revised = adapter.adapt(&graph).unwrap();
    let receipt = runtime.build(&revised, CancellationToken::new()).unwrap();
    for node in &receipt.nodes {
        let expected = if node.action_kind == "evidence" {
            ProjectNodeStatus::Executed
        } else {
            ProjectNodeStatus::CacheHit
        };
        assert_eq!(
            node.status, expected,
            "unexpected status for {}",
            node.target
        );
    }
    assert_eq!(backend.calls(), 4);
}

#[test]
fn builtin_evidence_module_is_excluded_from_authored_revision() {
    let fixture = Fixture::new();
    let authored = veac_evidence::build_evidence_root_path(
        &fixture.source,
        std::path::Path::new("evidence.veac"),
    )
    .unwrap();
    assert_eq!(authored.sources.len(), 2);
    assert!(!authored
        .sources
        .contains_key(veac_evidence::EVIDENCE_MODULE_ID));

    let plan = fixture.adapter().adapt(&graph()).unwrap();
    let (_, revision) = evidence_source(evidence_action(&plan));
    assert_eq!(revision.module_count, 2);
    assert_eq!(revision.modules, ["evidence-helper.veac", "evidence.veac"]);
    assert_eq!(
        revision.source_graph_sha256,
        authored.source_revision.source_graph_sha256
    );
}

#[test]
fn profile_instances_share_the_same_evidence_graph_snapshot() {
    let fixture = Fixture::new();
    let mut graph = graph();
    let mut duplicate = graph
        .instances
        .iter()
        .find(|instance| instance.id.as_str() == "final")
        .unwrap()
        .clone();
    duplicate.id = TargetInstanceId::from("final-master");
    duplicate.profile = Some(ProfileId::from("master"));
    duplicate.deliveries.clear();
    graph.instances.push(duplicate);
    let mut edge = graph.edges.last().unwrap().clone();
    edge.consumer = TargetInstanceId::from("final-master");
    graph.edges.push(edge);
    graph
        .build_order
        .push(TargetInstanceId::from("final-master"));

    let plan = fixture.adapter().adapt(&graph).unwrap();
    let snapshots = plan
        .graph
        .topology()
        .iter()
        .filter_map(|id| match plan.graph.node(id).unwrap().action() {
            ProjectAction::Evidence {
                contract,
                source_graph,
                ..
            } => Some((contract, source_graph)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(snapshots.len(), 2);
    assert_eq!(snapshots[0], snapshots[1]);
}

fn evidence_action(plan: &ProjectBuildPlan) -> &ProjectAction {
    plan.graph
        .topology()
        .iter()
        .map(|id| plan.graph.node(id).unwrap().action())
        .find(|action| matches!(action, ProjectAction::Evidence { .. }))
        .unwrap()
}

fn evidence_source(
    action: &ProjectAction,
) -> (
    &veac_build::ProjectFileSnapshot,
    &veac_build::ProjectSourceGraphRevision,
) {
    let ProjectAction::Evidence {
        contract,
        source_graph,
        ..
    } = action
    else {
        panic!("expected evidence action")
    };
    (contract, source_graph)
}

fn revise_evidence_helper(fixture: &Fixture) {
    std::fs::write(
        fixture.source.join("evidence-helper.veac"),
        REVISED_EVIDENCE_HELPER,
    )
    .unwrap();
}
