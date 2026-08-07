use super::*;

#[test]
fn contract_actions_construct_v6_descriptions_and_keyed_entities() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let canvas = canvas(&mut graph);
    let rate = frame_rate(&mut graph);
    let range = during(&mut graph);
    let resource = resource(&mut graph, "hero", "assets/hero.png");
    let project = project(&mut graph, "project");
    let sequence = sequence(&mut graph, "sequence");
    let layer = layer(&mut graph, "visual");
    let item = visual_item(&mut graph, "solid", 0);

    for (value, expected) in [
        (canvas, DomainType::Canvas),
        (rate, DomainType::FrameRate),
        (range, DomainType::TimeRange),
        (resource, DomainType::Resource),
        (project, DomainType::Project),
        (sequence, DomainType::Sequence),
        (layer, DomainType::Layer),
        (item, DomainType::Item),
    ] {
        assert_eq!(domain(&value).domain_type(), expected);
    }
}

#[test]
fn single_attachments_and_handle_entry_preserve_canonical_order() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let first_item = visual_item(&mut graph, "a", 0);
    let second_item = visual_item(&mut graph, "b", 1);
    let first_layer = layer(&mut graph, "one");
    let first_layer = attach(
        &mut graph,
        DomainOperationId::LayerWithItem,
        first_layer,
        first_item,
    );
    let second_layer = layer(&mut graph, "two");
    let second_layer = attach(
        &mut graph,
        DomainOperationId::LayerWithItem,
        second_layer,
        second_item,
    );
    let sequence = sequence(&mut graph, "intro");
    let sequence = attach(
        &mut graph,
        DomainOperationId::SequenceWithLayer,
        sequence,
        first_layer,
    );
    let sequence = attach(
        &mut graph,
        DomainOperationId::SequenceWithLayer,
        sequence,
        second_layer,
    );
    let entry = sequence.clone();
    let project = project(&mut graph, "root");
    let project = attach(
        &mut graph,
        DomainOperationId::ProjectWithSequence,
        project,
        sequence,
    );
    let project = evaluate(
        &mut graph,
        DomainOperationId::ProjectEntry,
        vec![project, entry],
    );
    let frozen = freeze(graph, &project);
    assert_eq!(frozen.entity_count(), 6);
    assert_eq!(frozen.entry_key(), Some("intro"));
    assert_eq!(frozen.root_logical_key(), ["root"]);
    assert_eq!(
        frozen.operation(domain(&project)),
        Some(DomainOperationId::ProjectEntry)
    );
    assert_eq!(frozen.operands(domain(&project)).unwrap().len(), 2);
    assert!(frozen.logical_bytes() > 0);
}
