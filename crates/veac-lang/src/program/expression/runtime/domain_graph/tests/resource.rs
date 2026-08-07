use super::*;
use crate::program::expression::runtime::domain_graph::validation;

#[test]
fn source_reference_keeps_resource_unowned_until_project_attachment() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let image = resource(&mut graph, "hero", "assets/hero.png");
    let image_node = validation::node(&graph, &image, DomainType::Resource, &(1..2)).unwrap();
    let source = evaluate(
        &mut graph,
        DomainOperationId::SourceMedia,
        vec![image.clone()],
    );
    assert!(graph.state.nodes[image_node.0].owner.is_none());
    assert_eq!(domain(&source).domain_type(), DomainType::Source);

    let root = complete(&mut graph);
    let root = attach(
        &mut graph,
        DomainOperationId::ProjectWithResource,
        root,
        image.clone(),
    );
    let frozen = freeze(graph, &root);
    assert_eq!(
        frozen.logical_key(domain(&image)).unwrap(),
        ["launch", "hero"]
    );
    assert_eq!(
        frozen.operation(domain(&source)),
        Some(DomainOperationId::SourceMedia)
    );
    assert_eq!(frozen.entity_count(), 5);
}

#[test]
fn freeze_rejects_an_unattached_resource() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let _orphan = resource(&mut graph, "orphan", "assets/orphan.png");
    let root = complete(&mut graph);
    assert_eq!(
        graph.freeze(&root, 1..2).unwrap_err().code(),
        "DOMAIN_PROJECT_DISCONNECTED"
    );
}

#[test]
fn repeated_attachment_is_generic_and_duplicate_keys_are_atomic() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let first = resource(&mut graph, "first", "assets/first.png");
    let second = resource(&mut graph, "second", "assets/second.png");
    let root = complete(&mut graph);
    let root = attach(
        &mut graph,
        DomainOperationId::ProjectWithResource,
        root,
        first,
    );
    let root = attach(
        &mut graph,
        DomainOperationId::ProjectWithResource,
        root,
        second,
    );
    let frozen = freeze(graph, &root);
    assert_eq!(
        frozen
            .root_entity()
            .children()
            .filter(|child| child.domain_type() == Some(DomainType::Resource))
            .count(),
        2
    );

    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let first = resource(&mut graph, "same", "assets/first.png");
    let second = resource(&mut graph, "same", "assets/second.png");
    let root = project(&mut graph, "project");
    let root = attach(
        &mut graph,
        DomainOperationId::ProjectWithResource,
        root,
        first,
    );
    let before = graph.record_count();
    let error = graph
        .evaluate(
            DomainOperationId::ProjectWithResource.opcode(),
            vec![root, second],
            1..2,
        )
        .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_DUPLICATE_KEY");
    assert_eq!(graph.record_count(), before);
}

#[test]
fn graph_handles_fail_closed_for_cross_graph_forged_and_stale_references() {
    let (registry, first_budget) = standard();
    let second_budget = crate::program::expression::ExecutionBudget::default();
    let mut first = DomainGraphTransaction::new(&registry, &first_budget);
    let foreign = resource(&mut first, "foreign", "assets/foreign.png");
    let mut second = DomainGraphTransaction::new(&registry, &second_budget);
    assert_eq!(
        second
            .evaluate(DomainOperationId::SourceMedia.opcode(), vec![foreign], 1..2)
            .unwrap_err()
            .code(),
        "DOMAIN_CROSS_GRAPH"
    );

    let mut graph = DomainGraphTransaction::new(&registry, &first_budget);
    let canvas = canvas(&mut graph);
    let forged = Value::Domain(Arc::new(
        crate::program::expression::DomainValue::from_arena(
            graph.scope.clone(),
            domain(&canvas).arena_slot(),
            DomainType::Resource,
        ),
    ));
    assert_eq!(
        graph
            .evaluate(DomainOperationId::SourceMedia.opcode(), vec![forged], 1..2)
            .unwrap_err()
            .code(),
        "DOMAIN_HANDLE_FORGED"
    );

    let mut graph = DomainGraphTransaction::new(&registry, &first_budget);
    let stale = resource(&mut graph, "stale", "assets/stale.png");
    let node = validation::node(&graph, &stale, DomainType::Resource, &(1..2)).unwrap();
    graph.state.nodes[node.0].latest_slot += 1;
    assert_eq!(
        graph
            .evaluate(DomainOperationId::SourceMedia.opcode(), vec![stale], 1..2)
            .unwrap_err()
            .code(),
        "DOMAIN_HANDLE_STALE"
    );
}
