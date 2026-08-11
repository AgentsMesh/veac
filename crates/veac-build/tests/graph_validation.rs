mod support;

use veac_build::*;

use support::{input, node, output, Action};

#[test]
fn graph_topology_and_digest_are_insertion_order_independent() {
    let graph = diamond(["root", "left", "right", "join"]);
    let reversed = diamond(["join", "right", "left", "root"]);
    let ids = graph
        .topology()
        .iter()
        .map(NodeId::as_str)
        .collect::<Vec<_>>();

    assert_eq!(ids, ["root", "left", "right", "join"]);
    assert_eq!(graph.digest(), reversed.digest());
    assert_eq!(graph.len(), 4);
    assert!(!graph.is_empty());
    assert_eq!(
        graph.node(&support::id("left")).unwrap().id().as_str(),
        "left"
    );
    assert_eq!(
        graph
            .node(&support::id("left"))
            .unwrap()
            .output_names()
            .count(),
        1
    );
}

#[test]
fn graph_digest_changes_with_action_or_resources() {
    let baseline = single(Action::new("one"), ResourceClaim::default());
    let mut revised = Action::new("one");
    revised.revision = 2;
    let revised = single(revised, ResourceClaim::default());
    let resourced = single(Action::new("one"), ResourceClaim::new(1, 64, 0));

    assert_ne!(baseline.digest(), revised.digest());
    assert_ne!(baseline.digest(), resourced.digest());
    assert_eq!(
        resourced
            .node(&support::id("one"))
            .unwrap()
            .resources_claimed(),
        ResourceClaim::new(1, 64, 0)
    );
}

#[test]
fn duplicate_and_dangling_contracts_are_rejected() {
    let mut duplicate = GraphBuilder::new();
    duplicate.add_node(node("one")).unwrap();
    assert!(duplicate.add_node(node("one")).is_err());

    let source = node("missing");
    let source_ref = source.output_ref(&output());
    let dangling = node("consumer").bind(&input(), &source_ref);
    assert_validation_error([dangling], "missing node");

    let producer = node("producer");
    let undeclared = producer.output_ref(&OutputSlot::new("other").unwrap());
    let consumer = node("consumer").bind(&input(), &undeclared);
    assert_validation_error([producer, consumer], "undeclared output");
}

#[test]
fn duplicate_ports_self_edges_and_cycles_are_rejected() {
    let duplicate_output = node("bad").output(&output());
    assert_validation_error([duplicate_output], "output 'artifact'");

    let source = node("source");
    let reference = source.output_ref(&output());
    let duplicate_input = node("bad")
        .bind(&input(), &reference)
        .bind(&input(), &reference);
    assert_validation_error([source, duplicate_input], "input role 'input'");

    let self_node = node("self");
    let self_ref = self_node.output_ref(&output());
    assert_validation_error([self_node.bind(&input(), &self_ref)], "depend on itself");

    let a = node("a");
    let b = node("b");
    let a_ref = a.output_ref(&output());
    let b_ref = b.output_ref(&output());
    assert_validation_error(
        [a.bind(&input(), &b_ref), b.bind(&input(), &a_ref)],
        "contains a cycle",
    );
}

fn single(action: Action, resources: ResourceClaim) -> ValidatedGraph<Action> {
    let mut builder = GraphBuilder::new();
    builder
        .add_node(
            NodeSpec::new(support::id("one"), action)
                .output(&output())
                .resources(resources),
        )
        .unwrap();
    builder.validate().unwrap()
}

fn diamond(order: [&str; 4]) -> ValidatedGraph<Action> {
    let root = node("root");
    let root_ref = root.output_ref(&output());
    let left = node("left").bind(&input(), &root_ref);
    let right = node("right").bind(&input(), &root_ref);
    let left_ref = left.output_ref(&output());
    let right_ref = right.output_ref(&output());
    let join = node("join")
        .bind(&InputSlot::new("left").unwrap(), &left_ref)
        .bind(&InputSlot::new("right").unwrap(), &right_ref);
    let mut nodes = [root, left, right, join]
        .into_iter()
        .map(|node| (node.id().to_string(), node))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut builder = GraphBuilder::new();
    for id in order {
        builder.add_node(nodes.remove(id).unwrap()).unwrap();
    }
    builder.validate().unwrap()
}

fn assert_validation_error<const N: usize>(nodes: [NodeSpec<Action>; N], fragment: &str) {
    let mut builder = GraphBuilder::new();
    for node in nodes {
        builder.add_node(node).unwrap();
    }
    let error = builder.validate().unwrap_err();
    assert!(error.to_string().contains(fragment), "{error}");
}
