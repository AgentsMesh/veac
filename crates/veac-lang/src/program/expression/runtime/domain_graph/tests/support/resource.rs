use super::*;

pub(in crate::program::expression::runtime::domain_graph) fn content_identity(
    graph: &mut DomainGraphTransaction<'_>,
) -> Value {
    evaluate(graph, DomainOperationId::Sha256, vec![text(DIGEST)])
}

pub(in crate::program::expression::runtime::domain_graph) fn resource(
    graph: &mut DomainGraphTransaction<'_>,
    key: &str,
    path: &str,
) -> Value {
    let location = evaluate(graph, DomainOperationId::ResourceFile, vec![text(path)]);
    let identity = content_identity(graph);
    evaluate(
        graph,
        DomainOperationId::ImageResource,
        vec![identifier(key), location, identity],
    )
}
