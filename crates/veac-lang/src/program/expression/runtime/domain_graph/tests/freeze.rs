use super::super::state::NodeId;
use super::super::validation;
use super::*;

#[test]
fn successful_freeze_publishes_closed_connected_project() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let root = complete(&mut graph);
    let frozen = freeze(graph, &root);
    assert_eq!(frozen.root().domain_type(), DomainType::Project);
    assert_eq!(frozen.root_logical_key(), ["launch"]);
    assert_eq!(frozen.entry_key(), Some("intro"));
    assert_eq!(frozen.entity_count(), 4);
}

#[test]
fn logical_keys_use_explicit_identifiers_instead_of_ordinals() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let first = sequence(&mut graph, "second-authored");
    let second = sequence(&mut graph, "first-authored");
    let first_handle = first.clone();
    let second_handle = second.clone();
    let root = project(&mut graph, "project");
    let root = attach(
        &mut graph,
        DomainOperationId::ProjectWithSequence,
        root,
        first,
    );
    let root = attach(
        &mut graph,
        DomainOperationId::ProjectWithSequence,
        root,
        second.clone(),
    );
    let root = evaluate(
        &mut graph,
        DomainOperationId::ProjectEntry,
        vec![root, second],
    );
    let frozen = freeze(graph, &root);
    assert_eq!(
        frozen.logical_key(domain(&first_handle)).unwrap(),
        ["project", "second-authored"]
    );
    assert_eq!(
        frozen.logical_key(domain(&second_handle)).unwrap(),
        ["project", "first-authored"]
    );
}

#[test]
fn freeze_rejects_missing_or_corrupt_entry_reference() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let child = sequence(&mut graph, "intro");
    let root = project(&mut graph, "project");
    let root = attach(
        &mut graph,
        DomainOperationId::ProjectWithSequence,
        root,
        child,
    );
    assert_eq!(
        graph.freeze(&root, 1..2).unwrap_err().code(),
        "DOMAIN_PROJECT_ENTRY_REQUIRED"
    );

    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let root = complete(&mut graph);
    let root_node = validation::node(&graph, &root, DomainType::Project, &(1..2)).unwrap();
    graph.state.nodes[root_node.0].entry = Some(NodeId(usize::MAX));
    assert_eq!(
        graph.freeze(&root, 1..2).unwrap_err().code(),
        "DOMAIN_PROJECT_ENTRY_INVALID"
    );
}

#[test]
fn freeze_rejects_disconnected_entities_and_invalid_roots() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let _orphan = sequence(&mut graph, "orphan");
    let root = complete(&mut graph);
    assert_eq!(
        graph.freeze(&root, 1..2).unwrap_err().code(),
        "DOMAIN_PROJECT_DISCONNECTED"
    );

    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let not_project = sequence(&mut graph, "sequence");
    assert_eq!(
        graph.freeze(&not_project, 3..4).unwrap_err().code(),
        "DOMAIN_ROOT_INVALID"
    );
}

#[test]
fn freeze_rejects_stale_and_foreign_project_handles() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let sequence = sequence(&mut graph, "entry");
    let stale = project(&mut graph, "project");
    let current = evaluate(
        &mut graph,
        DomainOperationId::ProjectEntry,
        vec![stale.clone(), sequence],
    );
    assert_eq!(
        graph.freeze(&stale, 1..2).unwrap_err().code(),
        "DOMAIN_ROOT_INVALID"
    );

    let graph = DomainGraphTransaction::new(&registry, &budget);
    assert_eq!(
        graph.freeze(&current, 1..2).unwrap_err().code(),
        "DOMAIN_ROOT_INVALID"
    );
}
