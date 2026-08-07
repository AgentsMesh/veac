use super::*;

pub(in crate::program::expression::runtime::domain_graph) fn project(
    graph: &mut DomainGraphTransaction<'_>,
    key: &str,
) -> Value {
    let settings = project_settings(graph);
    evaluate(
        graph,
        DomainOperationId::Project,
        vec![identifier(key), settings],
    )
}

pub(in crate::program::expression::runtime::domain_graph) fn sequence(
    graph: &mut DomainGraphTransaction<'_>,
    key: &str,
) -> Value {
    let settings = sequence_settings(graph);
    evaluate(
        graph,
        DomainOperationId::Sequence,
        vec![identifier(key), text(key), settings],
    )
}

pub(in crate::program::expression::runtime::domain_graph) fn layer(
    graph: &mut DomainGraphTransaction<'_>,
    key: &str,
) -> Value {
    let placement = evaluate(graph, DomainOperationId::PlacementFree, vec![]);
    let state = track_state(graph);
    let routing = evaluate(graph, DomainOperationId::TrackRoutingDefault, vec![]);
    evaluate(
        graph,
        DomainOperationId::VisualLayer,
        vec![
            identifier(key),
            Value::Integer(0),
            placement,
            state,
            routing,
        ],
    )
}

pub(in crate::program::expression::runtime::domain_graph) fn visual_item(
    graph: &mut DomainGraphTransaction<'_>,
    key: &str,
    start: i128,
) -> Value {
    let state = evaluate(graph, DomainOperationId::ItemEnabled, vec![]);
    let range = evaluate(graph, DomainOperationId::During, vec![time(start), time(2)]);
    let source = generated_source(graph);
    let timing = evaluate(graph, DomainOperationId::SourceTimingNative, vec![]);
    evaluate(
        graph,
        DomainOperationId::Item,
        vec![identifier(key), state, range, source, timing],
    )
}

pub(in crate::program::expression::runtime::domain_graph) fn attach(
    graph: &mut DomainGraphTransaction<'_>,
    operation: DomainOperationId,
    parent: Value,
    child: Value,
) -> Value {
    evaluate(graph, operation, vec![parent, child])
}

pub(in crate::program::expression::runtime::domain_graph) fn complete(
    graph: &mut DomainGraphTransaction<'_>,
) -> Value {
    let item = visual_item(graph, "background", 0);
    let layer = layer(graph, "visual");
    let layer = attach(graph, DomainOperationId::LayerWithItem, layer, item);
    let sequence = sequence(graph, "intro");
    let sequence = attach(graph, DomainOperationId::SequenceWithLayer, sequence, layer);
    let entry = sequence.clone();
    let project = project(graph, "launch");
    let project = attach(
        graph,
        DomainOperationId::ProjectWithSequence,
        project,
        sequence,
    );
    evaluate(graph, DomainOperationId::ProjectEntry, vec![project, entry])
}
