mod support;

use std::collections::BTreeMap;

use veac_build::*;

use support::{input, output, Action, FakeExecutor};

#[test]
fn bounded_scheduler_honors_fan_out_and_fan_in_barriers() {
    let graph = diamond_graph();
    let executor = FakeExecutor::default();
    let receipt = support::scheduler(2)
        .run(&graph, &executor, &NullBuildCache, CancellationToken::new())
        .unwrap();

    assert_eq!(receipt.outcome, BuildOutcome::Succeeded);
    assert_eq!(executor.stats.max_active(), 2);
    assert!(receipt
        .nodes
        .iter()
        .all(|node| node.status == NodeStatus::Executed));
    assert_eq!(
        receipt
            .nodes
            .iter()
            .map(|node| node.node_id.as_str())
            .collect::<Vec<_>>(),
        ["join", "left", "right", "root"]
    );
    assert_eq!(receipt.graph_digest, graph.digest().clone());
    assert!(receipt.node(&support::id("absent")).is_none());

    let events = executor.stats.events();
    let start_join = position(&events, "start:join");
    assert!(start_join > position(&events, "finish:left"));
    assert!(start_join > position(&events, "finish:right"));
    assert!(position(&events, "start:left") > position(&events, "finish:root"));
    assert!(position(&events, "start:right") > position(&events, "finish:root"));
}

#[test]
fn jobs_bound_independent_work_and_receipts_remain_sorted() {
    let mut builder = GraphBuilder::new();
    for name in ["delta", "alpha", "charlie", "bravo"] {
        let mut action = Action::new(name);
        action.delay_ms = 20;
        builder
            .add_node(NodeSpec::new(support::id(name), action).output(&output()))
            .unwrap();
    }
    let graph = builder.validate().unwrap();
    let executor = FakeExecutor::default();
    let receipt = support::scheduler(2)
        .run(&graph, &executor, &NullBuildCache, CancellationToken::new())
        .unwrap();

    assert_eq!(executor.stats.max_active(), 2);
    assert_eq!(
        receipt
            .nodes
            .iter()
            .map(|node| node.node_id.as_str())
            .collect::<Vec<_>>(),
        ["alpha", "bravo", "charlie", "delta"]
    );
}

#[test]
fn repeated_uncached_builds_produce_identical_receipts() {
    let graph = diamond_graph();
    let scheduler = support::scheduler(2);
    let first = scheduler
        .run(
            &graph,
            &FakeExecutor::default(),
            &NullBuildCache,
            CancellationToken::new(),
        )
        .unwrap();
    let second = scheduler
        .run(
            &graph,
            &FakeExecutor::default(),
            &NullBuildCache,
            CancellationToken::new(),
        )
        .unwrap();

    assert_eq!(first, second);
}

#[test]
fn resource_admission_serializes_claims_even_with_free_jobs() {
    let mut builder = GraphBuilder::new();
    for name in ["first", "second"] {
        let mut action = Action::new(name);
        action.delay_ms = 20;
        builder
            .add_node(
                NodeSpec::new(support::id(name), action)
                    .output(&output())
                    .resources(ResourceClaim::new(1, 400, 1)),
            )
            .unwrap();
    }
    let graph = builder.validate().unwrap();
    let limits = BuildLimits::new(2, ResourceClaim::new(2, 512, 1)).unwrap();
    let scheduler = BuildScheduler::new(limits);
    let executor = FakeExecutor::default();
    let receipt = scheduler
        .run(&graph, &executor, &NullBuildCache, CancellationToken::new())
        .unwrap();

    assert_eq!(scheduler.limits(), limits);
    assert_eq!(receipt.outcome, BuildOutcome::Succeeded);
    assert_eq!(executor.stats.max_active(), 1);
}

#[test]
fn impossible_resource_claim_is_rejected_before_any_work() {
    let graph = {
        let mut builder = GraphBuilder::new();
        builder
            .add_node(support::node("gpu-heavy").resources(ResourceClaim::new(1, 0, 2)))
            .unwrap();
        builder.validate().unwrap()
    };
    let executor = FakeExecutor::default();
    let error = support::scheduler(2)
        .run(&graph, &executor, &NullBuildCache, CancellationToken::new())
        .unwrap_err();

    assert_eq!(error.kind(), BuildErrorKind::ResourceLimit);
    assert_eq!(executor.stats.count("gpu-heavy"), 0);
}

fn diamond_graph() -> ValidatedGraph<Action> {
    let mut actions = BTreeMap::new();
    for (name, delay) in [("root", 5), ("left", 25), ("right", 25), ("join", 0)] {
        let mut action = Action::new(name);
        action.delay_ms = delay;
        actions.insert(name, action);
    }
    let root =
        NodeSpec::new(support::id("root"), actions.remove("root").unwrap()).output(&output());
    let root_ref = root.output_ref(&output());
    let left = NodeSpec::new(support::id("left"), actions.remove("left").unwrap())
        .output(&output())
        .bind(&input(), &root_ref);
    let right = NodeSpec::new(support::id("right"), actions.remove("right").unwrap())
        .output(&output())
        .bind(&input(), &root_ref);
    let join = NodeSpec::new(support::id("join"), actions.remove("join").unwrap())
        .output(&output())
        .bind(
            &InputSlot::new("left").unwrap(),
            &left.output_ref(&output()),
        )
        .bind(
            &InputSlot::new("right").unwrap(),
            &right.output_ref(&output()),
        );
    let mut builder = GraphBuilder::new();
    for node in [join, right, root, left] {
        builder.add_node(node).unwrap();
    }
    builder.validate().unwrap()
}

fn position(events: &[String], expected: &str) -> usize {
    events.iter().position(|event| event == expected).unwrap()
}
