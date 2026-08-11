mod support;

use veac_build::*;

use support::{node, output, Action, FakeExecutor};

#[test]
fn order_only_dependencies_gate_execution_without_becoming_inputs() {
    let mut first_action = Action::new("first");
    first_action.delay_ms = 20;
    let first = NodeSpec::new(support::id("first"), first_action).output(&output());
    let second = node("second").after(first.id());
    let mut builder = GraphBuilder::new();
    builder.add_node(second).unwrap();
    builder.add_node(first).unwrap();
    let graph = builder.validate().unwrap();
    let executor = FakeExecutor::default();
    let receipt = support::scheduler(2)
        .run(&graph, &executor, &NullBuildCache, CancellationToken::new())
        .unwrap();

    let events = executor.stats.events();
    let finished = events
        .iter()
        .position(|event| event == "finish:first")
        .unwrap();
    let started = events
        .iter()
        .position(|event| event == "start:second")
        .unwrap();
    assert!(started > finished);
    assert_eq!(receipt.outcome, BuildOutcome::Succeeded);
}

#[test]
fn order_only_dependencies_change_graph_but_not_computation_identity() {
    let independent = two_nodes(false);
    let ordered = two_nodes(true);
    let cache = MemoryBuildCache::new();
    support::scheduler(2)
        .run(
            &independent,
            &FakeExecutor::default(),
            &cache,
            CancellationToken::new(),
        )
        .unwrap();
    let receipt = support::scheduler(2)
        .run(
            &ordered,
            &FakeExecutor::default(),
            &cache,
            CancellationToken::new(),
        )
        .unwrap();

    assert_ne!(independent.digest(), ordered.digest());
    assert!(receipt
        .nodes
        .iter()
        .all(|node| node.status == NodeStatus::CacheHit));
}

#[test]
fn malformed_order_dependencies_are_rejected() {
    let one = node("one");
    let duplicate = node("two").after(one.id()).after(one.id());
    assert_error([one, duplicate], "order dependency");

    let missing = node("one").after(&support::id("missing"));
    assert_error([missing], "missing node dependency");

    let self_edge = node("one").after(&support::id("one"));
    assert_error([self_edge], "depend on itself");
}

fn two_nodes(ordered: bool) -> ValidatedGraph<Action> {
    let one = node("one");
    let mut two = node("two");
    if ordered {
        two = two.after(one.id());
    }
    let mut builder = GraphBuilder::new();
    builder.add_node(two).unwrap();
    builder.add_node(one).unwrap();
    builder.validate().unwrap()
}

fn assert_error<const N: usize>(nodes: [NodeSpec<Action>; N], fragment: &str) {
    let mut builder = GraphBuilder::new();
    for node in nodes {
        builder.add_node(node).unwrap();
    }
    let error = builder.validate().unwrap_err();
    assert!(error.to_string().contains(fragment), "{error}");
}
