mod support;

use std::{thread, time::Duration};

use veac_build::*;

use support::{input, node, output, Action, Behavior, FakeExecutor};

#[test]
fn failed_node_blocks_descendants_and_cancels_independent_pending_work() {
    let graph = failure_graph(false);
    let executor = FakeExecutor::default();
    let receipt = support::scheduler(1)
        .run(&graph, &executor, &NullBuildCache, CancellationToken::new())
        .unwrap();

    assert_eq!(receipt.outcome, BuildOutcome::Failed);
    assert_eq!(status(&receipt, "a-fail"), NodeStatus::Failed);
    assert_eq!(status(&receipt, "child"), NodeStatus::Blocked);
    assert_eq!(status(&receipt, "grandchild"), NodeStatus::Blocked);
    assert_eq!(status(&receipt, "z-independent"), NodeStatus::Cancelled);
    assert_eq!(executor.stats.count("a-fail"), 1);
    assert_eq!(executor.stats.count("child"), 0);
    assert!(receipt
        .node(&support::id("child"))
        .unwrap()
        .message
        .as_deref()
        .unwrap()
        .contains("failed dependency"));
}

#[test]
fn concurrent_workers_observe_fail_fast_cancellation() {
    let graph = failure_graph(true);
    let executor = FakeExecutor::default();
    let token = CancellationToken::new();
    let receipt = support::scheduler(2)
        .run(&graph, &executor, &NullBuildCache, token.clone())
        .unwrap();

    assert!(token.is_cancelled());
    assert_eq!(receipt.outcome, BuildOutcome::Failed);
    assert_eq!(status(&receipt, "a-fail"), NodeStatus::Failed);
    assert_eq!(status(&receipt, "z-independent"), NodeStatus::Cancelled);
    assert_eq!(executor.stats.count("z-independent"), 1);
}

#[test]
fn pre_cancelled_build_never_admits_a_node() {
    let mut builder = GraphBuilder::new();
    builder.add_node(node("one")).unwrap();
    builder.add_node(node("two")).unwrap();
    let graph = builder.validate().unwrap();
    let token = CancellationToken::new();
    token.cancel();
    let executor = FakeExecutor::default();
    let receipt = support::scheduler(2)
        .run(&graph, &executor, &NullBuildCache, token)
        .unwrap();

    assert_eq!(receipt.outcome, BuildOutcome::Cancelled);
    assert!(receipt
        .nodes
        .iter()
        .all(|node| node.status == NodeStatus::Cancelled));
    assert_eq!(executor.stats.max_active(), 0);
}

#[test]
fn external_cancellation_is_visible_to_an_active_executor() {
    let mut action = Action::new("long-running");
    action.delay_ms = 100;
    action.observe_cancel = true;
    let mut builder = GraphBuilder::new();
    builder
        .add_node(NodeSpec::new(support::id("long-running"), action).output(&output()))
        .unwrap();
    let graph = builder.validate().unwrap();
    let token = CancellationToken::new();
    let trigger = token.clone();
    let canceller = thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        trigger.cancel();
    });
    let receipt = support::scheduler(1)
        .run(&graph, &FakeExecutor::default(), &NullBuildCache, token)
        .unwrap();
    canceller.join().unwrap();

    assert_eq!(receipt.outcome, BuildOutcome::Cancelled);
    assert_eq!(status(&receipt, "long-running"), NodeStatus::Cancelled);
}

#[test]
fn executor_cancellation_cancels_instead_of_blocking_descendants() {
    let mut action = Action::new("cancel");
    action.behavior = Behavior::Cancel;
    let root = NodeSpec::new(support::id("cancel"), action).output(&output());
    let child = node("child").bind(&input(), &root.output_ref(&output()));
    let mut builder = GraphBuilder::new();
    builder.add_node(root).unwrap();
    builder.add_node(child).unwrap();
    let graph = builder.validate().unwrap();
    let receipt = support::scheduler(1)
        .run(
            &graph,
            &FakeExecutor::default(),
            &NullBuildCache,
            CancellationToken::new(),
        )
        .unwrap();

    assert_eq!(receipt.outcome, BuildOutcome::Cancelled);
    assert_eq!(status(&receipt, "cancel"), NodeStatus::Cancelled);
    assert_eq!(status(&receipt, "child"), NodeStatus::Cancelled);
}

#[test]
fn executor_panics_and_wrong_outputs_become_stable_failures() {
    for (behavior, fragment) in [
        (Behavior::Panic, "executor panicked"),
        (Behavior::WrongOutput, "outputs that do not match"),
    ] {
        let mut action = Action::new("broken");
        action.behavior = behavior;
        let mut builder = GraphBuilder::new();
        builder
            .add_node(NodeSpec::new(support::id("broken"), action).output(&output()))
            .unwrap();
        let graph = builder.validate().unwrap();
        let receipt = support::scheduler(1)
            .run(
                &graph,
                &FakeExecutor::default(),
                &NullBuildCache,
                CancellationToken::new(),
            )
            .unwrap();

        assert_eq!(receipt.outcome, BuildOutcome::Failed);
        assert!(receipt.nodes[0]
            .message
            .as_deref()
            .unwrap()
            .contains(fragment));
    }
}

fn failure_graph(concurrent: bool) -> ValidatedGraph<Action> {
    let mut failure = Action::new("a-fail");
    failure.behavior = Behavior::Fail;
    failure.delay_ms = u64::from(concurrent) * 5;
    let root = NodeSpec::new(support::id("a-fail"), failure).output(&output());
    let child = node("child").bind(&input(), &root.output_ref(&output()));
    let grandchild = node("grandchild").bind(&input(), &child.output_ref(&output()));
    let mut independent = Action::new("z-independent");
    independent.delay_ms = u64::from(concurrent) * 100;
    independent.observe_cancel = concurrent;
    let independent = NodeSpec::new(support::id("z-independent"), independent).output(&output());
    let mut builder = GraphBuilder::new();
    for node in [root, child, grandchild, independent] {
        builder.add_node(node).unwrap();
    }
    builder.validate().unwrap()
}

fn status(receipt: &BuildReceipt, id: &str) -> NodeStatus {
    receipt.node(&support::id(id)).unwrap().status
}
