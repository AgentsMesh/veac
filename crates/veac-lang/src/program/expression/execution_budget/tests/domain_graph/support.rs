use super::*;

pub(super) const DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";

pub(super) struct Snapshot {
    pub(super) records: usize,
    pub(super) bytes: usize,
    pub(super) entities: usize,
}

pub(super) fn snapshot(
    graph: &DomainGraphTransaction<'_>,
    execution: &ExecutionBudget,
) -> Snapshot {
    Snapshot {
        records: graph.record_count(),
        bytes: used(execution, Resource::EmittedBytes),
        entities: used(execution, Resource::EmittedEntities),
    }
}

pub(super) fn assert_atomic_failure(
    graph: &DomainGraphTransaction<'_>,
    execution: &ExecutionBudget,
    error: crate::program::expression::ExpressionError,
    records: usize,
    bytes: usize,
    entities: usize,
) {
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert_eq!(graph.record_count(), records);
    assert_eq!(used(execution, Resource::EmittedBytes), bytes);
    assert_eq!(used(execution, Resource::EmittedEntities), entities);
}

pub(super) fn assert_tainted(graph: &mut DomainGraphTransaction<'_>) {
    let error = graph
        .evaluate(DomainOperationId::Canvas.opcode(), vec![], 2..3)
        .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_TRANSACTION_FAILED");
}

pub(super) fn evaluate_result(
    graph: &mut DomainGraphTransaction<'_>,
    operation: DomainOperationId,
    operands: Vec<Value>,
) -> Result<Value, crate::program::expression::ExpressionError> {
    graph.evaluate(operation.opcode(), operands, 1..2)
}

pub(super) fn evaluate(
    graph: &mut DomainGraphTransaction<'_>,
    operation: DomainOperationId,
    operands: Vec<Value>,
) -> Value {
    evaluate_result(graph, operation, operands).unwrap()
}

pub(super) fn length(value: i128) -> Value {
    Value::Length(super::super::super::super::ExactNumber::integer(value))
}

pub(super) fn time(value: i128) -> Value {
    Value::Time(super::super::super::super::ExactNumber::integer(value))
}

pub(super) fn text(value: &str) -> Value {
    Value::Text(Arc::from(value))
}

pub(super) fn list(kind: DomainType, values: Vec<Value>) -> Value {
    Value::list(super::super::super::super::ValueType::domain(kind), values).unwrap()
}

pub(super) fn canvas(graph: &mut DomainGraphTransaction<'_>) -> Value {
    evaluate(
        graph,
        DomainOperationId::Canvas,
        vec![length(1080), length(1920)],
    )
}

fn frame_rate(graph: &mut DomainGraphTransaction<'_>) -> Value {
    evaluate(
        graph,
        DomainOperationId::FrameRate,
        vec![Value::Integer(30), Value::Integer(1)],
    )
}

pub(super) fn sequence_operands(graph: &mut DomainGraphTransaction<'_>, key: &str) -> Vec<Value> {
    let canvas = canvas(graph);
    let frame_rate = frame_rate(graph);
    let settings = evaluate(
        graph,
        DomainOperationId::SequenceSettings,
        vec![canvas, frame_rate, Value::Integer(48_000)],
    );
    vec![identifier(key), text(key), settings]
}

pub(super) fn sequence(graph: &mut DomainGraphTransaction<'_>, key: &str) -> Value {
    let operands = sequence_operands(graph, key);
    evaluate(graph, DomainOperationId::Sequence, operands)
}

pub(super) fn visual_item(graph: &mut DomainGraphTransaction<'_>, key: &str, start: i128) -> Value {
    let state = evaluate(graph, DomainOperationId::ItemEnabled, vec![]);
    let range = evaluate(graph, DomainOperationId::During, vec![time(start), time(2)]);
    let generator = evaluate(graph, DomainOperationId::GeneratorTransparent, vec![]);
    let source = evaluate(graph, DomainOperationId::SourceGenerated, vec![generator]);
    let timing = evaluate(graph, DomainOperationId::SourceTimingNative, vec![]);
    evaluate(
        graph,
        DomainOperationId::Item,
        vec![identifier(key), state, range, source, timing],
    )
}

pub(super) fn template(graph: &mut DomainGraphTransaction<'_>) -> Value {
    let kind = evaluate(graph, DomainOperationId::SlotVideo, vec![]);
    let fill = evaluate(graph, DomainOperationId::FillTakeCenter, vec![]);
    let duration = evaluate(graph, DomainOperationId::SourceDurationAny, vec![]);
    let policy = evaluate(graph, DomainOperationId::TemplateTextLocked, vec![]);
    evaluate(
        graph,
        DomainOperationId::TemplateContract,
        vec![kind, fill, text("slot"), duration, policy],
    )
}
