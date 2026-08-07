use super::*;

pub(in crate::program::expression::runtime::domain_graph) fn template(
    graph: &mut DomainGraphTransaction<'_>,
) -> Value {
    let kind = evaluate(graph, DomainOperationId::SlotVideo, vec![]);
    let fill = evaluate(graph, DomainOperationId::FillTakeCenter, vec![]);
    let duration = evaluate(graph, DomainOperationId::SourceDurationAny, vec![]);
    let text_policy = evaluate(graph, DomainOperationId::TemplateTextLocked, vec![]);
    evaluate(
        graph,
        DomainOperationId::TemplateContract,
        vec![kind, fill, text("slot"), duration, text_policy],
    )
}

pub(in crate::program::expression::runtime::domain_graph) fn update_template(
    graph: &mut DomainGraphTransaction<'_>,
    item: Value,
) -> Value {
    let contract = template(graph);
    evaluate(
        graph,
        DomainOperationId::ItemWithTemplate,
        vec![item, contract],
    )
}
