use super::super::validation;
use super::*;

#[test]
fn plural_attachments_preserve_order_and_cover_owner_boundaries() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let first = visual_item(&mut graph, "first", 0);
    let second = visual_item(&mut graph, "second", 1);
    let base = layer(&mut graph, "cards");
    let current = evaluate(
        &mut graph,
        DomainOperationId::LayerWithItems,
        vec![base, list(DomainType::Item, vec![first, second])],
    );
    assert_eq!(
        child_keys(&graph, &current, DomainType::Layer),
        ["first", "second"]
    );

    let before = graph.record_count();
    let current = evaluate(
        &mut graph,
        DomainOperationId::LayerWithItems,
        vec![current, list(DomainType::Item, vec![])],
    );
    assert_eq!(graph.record_count(), before + 1);
    assert_eq!(
        child_keys(&graph, &current, DomainType::Layer),
        ["first", "second"]
    );

    let extra = layer(&mut graph, "empty");
    let timeline = sequence(&mut graph, "main");
    let timeline = evaluate(
        &mut graph,
        DomainOperationId::SequenceWithLayers,
        vec![timeline, list(DomainType::Layer, vec![current, extra])],
    );
    assert_eq!(
        child_keys(&graph, &timeline, DomainType::Sequence),
        ["cards", "empty"]
    );
    let project = project(&mut graph, "project");
    let project = evaluate(
        &mut graph,
        DomainOperationId::ProjectWithSequences,
        vec![project, list(DomainType::Sequence, vec![timeline])],
    );
    assert_eq!(child_keys(&graph, &project, DomainType::Project), ["main"]);
}

#[test]
fn duplicate_children_and_keys_fail_before_any_owner_changes() {
    for duplicate_handle in [true, false] {
        let (registry, budget) = standard();
        let mut graph = DomainGraphTransaction::new(&registry, &budget);
        let first = visual_item(&mut graph, "same", 0);
        let second = if duplicate_handle {
            first.clone()
        } else {
            visual_item(&mut graph, "same", 1)
        };
        let first_node = node(&graph, &first, DomainType::Item);
        let second_node = node(&graph, &second, DomainType::Item);
        let parent = layer(&mut graph, "target");
        let parent_node = node(&graph, &parent, DomainType::Layer);
        let before = graph.record_count();
        let error = graph
            .evaluate(
                DomainOperationId::LayerWithItems.opcode(),
                vec![parent, list(DomainType::Item, vec![first, second])],
                1..2,
            )
            .unwrap_err();
        let expected = if duplicate_handle {
            "DOMAIN_SINGLE_OWNERSHIP"
        } else {
            "DOMAIN_DUPLICATE_KEY"
        };
        assert_eq!(error.code(), expected);
        assert_eq!(graph.record_count(), before);
        assert!(graph.state.nodes[parent_node.0].children.is_empty());
        assert!(graph.state.nodes[first_node.0].owner.is_none());
        assert!(graph.state.nodes[second_node.0].owner.is_none());
        assert_eq!(
            graph
                .evaluate(DomainOperationId::Canvas.opcode(), vec![], 2..3)
                .unwrap_err()
                .code(),
            "DOMAIN_TRANSACTION_FAILED"
        );
    }
}

#[test]
fn owned_or_cross_graph_tail_cannot_partially_attach_a_prefix() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let owned = visual_item(&mut graph, "owned", 0);
    let owner = layer(&mut graph, "owner");
    attach(
        &mut graph,
        DomainOperationId::LayerWithItem,
        owner,
        owned.clone(),
    );
    let fresh = visual_item(&mut graph, "fresh", 1);
    let fresh_node = node(&graph, &fresh, DomainType::Item);
    let target = layer(&mut graph, "target");
    let target_node = node(&graph, &target, DomainType::Layer);
    let error = graph
        .evaluate(
            DomainOperationId::LayerWithItems.opcode(),
            vec![target, list(DomainType::Item, vec![fresh, owned])],
            1..2,
        )
        .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_SINGLE_OWNERSHIP");
    assert!(graph.state.nodes[target_node.0].children.is_empty());
    assert!(graph.state.nodes[fresh_node.0].owner.is_none());

    let first_budget = crate::program::expression::ExecutionBudget::default();
    let second_budget = crate::program::expression::ExecutionBudget::default();
    let mut first = DomainGraphTransaction::new(&registry, &first_budget);
    let foreign = visual_item(&mut first, "foreign", 0);
    let mut second = DomainGraphTransaction::new(&registry, &second_budget);
    let local = visual_item(&mut second, "local", 0);
    let local_node = node(&second, &local, DomainType::Item);
    let target = layer(&mut second, "target");
    let target_node = node(&second, &target, DomainType::Layer);
    let before = second.record_count();
    let error = Value::list(ValueType::domain(DomainType::Item), vec![local, foreign]).unwrap_err();
    assert_eq!(error.code(), "VALUE_DOMAIN_CROSS_GRAPH");
    assert_eq!(second.record_count(), before);
    assert!(second.state.nodes[target_node.0].children.is_empty());
    assert!(second.state.nodes[local_node.0].owner.is_none());
}

#[test]
fn empty_plural_update_still_invalidates_the_stale_parent_handle() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let stale = layer(&mut graph, "layer");
    let current = evaluate(
        &mut graph,
        DomainOperationId::LayerWithItems,
        vec![stale.clone(), list(DomainType::Item, vec![])],
    );
    assert_ne!(stale, current);
    let error = graph
        .evaluate(
            DomainOperationId::LayerWithItems.opcode(),
            vec![stale, list(DomainType::Item, vec![])],
            1..2,
        )
        .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_HANDLE_STALE");
}

fn node(
    graph: &DomainGraphTransaction<'_>,
    value: &Value,
    kind: DomainType,
) -> super::super::state::NodeId {
    validation::node(graph, value, kind, &(1..2)).unwrap()
}

fn child_keys<'a>(
    graph: &'a DomainGraphTransaction<'_>,
    value: &Value,
    kind: DomainType,
) -> Vec<&'a str> {
    let parent = node(graph, value, kind);
    graph.state.nodes[parent.0]
        .children
        .iter()
        .map(|child| graph.state.nodes[child.0].key.as_deref().unwrap())
        .collect()
}
