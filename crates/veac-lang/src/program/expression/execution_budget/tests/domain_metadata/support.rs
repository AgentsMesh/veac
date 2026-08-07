use super::*;

pub(super) struct Snapshot {
    pub(super) records: usize,
    pub(super) bytes: usize,
    pub(super) entities: usize,
}

pub(super) struct Attempt<'a> {
    pub(super) graph: DomainGraphTransaction<'a>,
    pub(super) result: Result<Value, crate::program::expression::ExpressionError>,
    pub(super) before: Snapshot,
}

pub(super) fn attach_layer<'a>(
    registry: &'a DomainOperationRegistry,
    budget: &'a ExecutionBudget,
    origin: &DomainOrigin,
) -> Attempt<'a> {
    let mut graph = DomainGraphTransaction::new(registry, budget);
    let sequence = sequence(&mut graph, origin);
    let layer = layer(&mut graph, origin);
    let before = Snapshot {
        records: graph.record_count(),
        bytes: used(budget, Resource::EmittedBytes),
        entities: used(budget, Resource::EmittedEntities),
    };
    let result = authored(
        &mut graph,
        DomainOperationId::SequenceWithLayer,
        vec![sequence, layer],
        origin,
    );
    Attempt {
        graph,
        result,
        before,
    }
}

fn sequence(graph: &mut DomainGraphTransaction<'_>, origin: &DomainOrigin) -> Value {
    let canvas = evaluate(
        graph,
        DomainOperationId::Canvas,
        vec![length(1920), length(1080)],
    );
    let rate = evaluate(
        graph,
        DomainOperationId::FrameRate,
        vec![Value::Integer(30), Value::Integer(1)],
    );
    let settings = evaluate(
        graph,
        DomainOperationId::SequenceSettings,
        vec![canvas, rate, Value::Integer(48_000)],
    );
    authored(
        graph,
        DomainOperationId::Sequence,
        vec![identifier("main"), Value::Text("Main".into()), settings],
        origin,
    )
    .unwrap()
}

fn layer(graph: &mut DomainGraphTransaction<'_>, origin: &DomainOrigin) -> Value {
    let placement = evaluate(graph, DomainOperationId::PlacementFree, vec![]);
    let playback = evaluate(graph, DomainOperationId::TrackPlaybackEnabled, vec![]);
    let audio = evaluate(graph, DomainOperationId::TrackAudioAudible, vec![]);
    let isolation = evaluate(graph, DomainOperationId::TrackIsolationNormal, vec![]);
    let editing = evaluate(graph, DomainOperationId::TrackEditingUnlocked, vec![]);
    let state = evaluate(
        graph,
        DomainOperationId::TrackState,
        vec![playback, audio, isolation, editing],
    );
    let routing = evaluate(graph, DomainOperationId::TrackRoutingDefault, vec![]);
    authored(
        graph,
        DomainOperationId::VisualLayer,
        vec![
            identifier("visual"),
            Value::Integer(0),
            placement,
            state,
            routing,
        ],
        origin,
    )
    .unwrap()
}

fn evaluate(
    graph: &mut DomainGraphTransaction<'_>,
    operation: DomainOperationId,
    operands: Vec<Value>,
) -> Value {
    graph.evaluate(operation.opcode(), operands, 1..2).unwrap()
}

fn authored(
    graph: &mut DomainGraphTransaction<'_>,
    operation: DomainOperationId,
    operands: Vec<Value>,
    origin: &DomainOrigin,
) -> Result<Value, crate::program::expression::ExpressionError> {
    graph.evaluate_authored(operation.opcode(), operands, 10..11, Some(origin.clone()))
}

fn length(value: i128) -> Value {
    Value::Length(crate::program::expression::ExactNumber::integer(value))
}
