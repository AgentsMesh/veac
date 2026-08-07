use super::super::validation;
use super::*;

#[test]
fn duplicate_keys_fail_before_attachment_mutates() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let first = sequence(&mut graph, "same");
    let second = sequence(&mut graph, "same");
    let root = project(&mut graph, "project");
    let root = attach(
        &mut graph,
        DomainOperationId::ProjectWithSequence,
        root,
        first,
    );
    let before = graph.record_count();
    let error = graph
        .evaluate(
            DomainOperationId::ProjectWithSequence.opcode(),
            vec![root, second],
            1..2,
        )
        .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_DUPLICATE_KEY");
    assert_eq!(graph.record_count(), before);
    assert_eq!(
        graph
            .freeze(&identifier("not-a-root"), 1..2)
            .unwrap_err()
            .code(),
        "DOMAIN_TRANSACTION_FAILED"
    );
}

#[test]
fn one_entity_cannot_be_attached_twice() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let child = sequence(&mut graph, "child");
    let first = project(&mut graph, "first");
    attach(
        &mut graph,
        DomainOperationId::ProjectWithSequence,
        first,
        child.clone(),
    );
    let second = project(&mut graph, "second");
    let error = graph
        .evaluate(
            DomainOperationId::ProjectWithSequence.opcode(),
            vec![second, child],
            1..2,
        )
        .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_SINGLE_OWNERSHIP");
}

#[test]
fn attached_container_topology_is_closed() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let child = layer(&mut graph, "layer");
    let child_handle = child.clone();
    let parent = sequence(&mut graph, "sequence");
    attach(
        &mut graph,
        DomainOperationId::SequenceWithLayer,
        parent,
        child,
    );
    let item = visual_item(&mut graph, "late", 0);
    let error = graph
        .evaluate(
            DomainOperationId::LayerWithItem.opcode(),
            vec![child_handle, item],
            1..2,
        )
        .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_TOPOLOGY_CLOSED");
}

#[test]
fn stale_parent_handle_cannot_branch_immutable_topology() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let stale = layer(&mut graph, "layer");
    let first_item = visual_item(&mut graph, "first", 0);
    let current = attach(
        &mut graph,
        DomainOperationId::LayerWithItem,
        stale.clone(),
        first_item,
    );
    assert_ne!(stale, current);
    let late = visual_item(&mut graph, "late", 0);
    let error = graph
        .evaluate(
            DomainOperationId::LayerWithItem.opcode(),
            vec![stale, late],
            1..2,
        )
        .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_HANDLE_STALE");
}

#[test]
fn ancestry_cycle_detection_is_total_even_for_corrupt_state() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let first = sequence(&mut graph, "first");
    let second = sequence(&mut graph, "second");
    let first = validation::node(&graph, &first, DomainType::Sequence, &(1..2)).unwrap();
    let second = validation::node(&graph, &second, DomainType::Sequence, &(1..2)).unwrap();
    graph.state.nodes[first.0].owner = Some(second);
    assert!(graph.state.would_cycle(first, second));
    graph.state.nodes[second.0].owner = Some(first);
    assert!(graph.state.would_cycle(first, first));
}
