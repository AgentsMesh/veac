pub(super) use super::*;

#[path = "relation/construction.rs"]
mod construction;
#[path = "relation/handles.rs"]
mod handles;
#[path = "relation/topology.rs"]
mod topology;
#[path = "relation/topology_corruption.rs"]
mod topology_corruption;

fn group(graph: &mut DomainGraphTransaction<'_>, key: &str, members: Vec<Value>) -> Value {
    evaluate(
        graph,
        DomainOperationId::RelationGroup,
        vec![identifier(key), list(DomainType::Item, members)],
    )
}

fn layer_with(graph: &mut DomainGraphTransaction<'_>, key: &str, members: Vec<Value>) -> Value {
    let mut layer = layer(graph, key);
    for member in members {
        layer = attach(graph, DomainOperationId::LayerWithItem, layer, member);
    }
    layer
}

fn sequence_with(
    graph: &mut DomainGraphTransaction<'_>,
    key: &str,
    layers: Vec<Value>,
    relation: Option<Value>,
) -> Value {
    let mut sequence = sequence(graph, key);
    for layer in layers {
        sequence = attach(graph, DomainOperationId::SequenceWithLayer, sequence, layer);
    }
    if let Some(relation) = relation {
        sequence = attach(
            graph,
            DomainOperationId::SequenceWithRelation,
            sequence,
            relation,
        );
    }
    sequence
}

fn project_with(
    graph: &mut DomainGraphTransaction<'_>,
    sequences: Vec<Value>,
    entry: Value,
) -> Value {
    let mut project = project(graph, "project");
    for sequence in sequences {
        project = attach(
            graph,
            DomainOperationId::ProjectWithSequence,
            project,
            sequence,
        );
    }
    evaluate(graph, DomainOperationId::ProjectEntry, vec![project, entry])
}
