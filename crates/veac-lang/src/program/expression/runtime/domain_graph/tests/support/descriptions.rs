use super::*;

pub(in crate::program::expression::runtime::domain_graph) fn canvas(
    graph: &mut DomainGraphTransaction<'_>,
) -> Value {
    evaluate(
        graph,
        DomainOperationId::Canvas,
        vec![length(1080), length(1920)],
    )
}

pub(in crate::program::expression::runtime::domain_graph) fn frame_rate(
    graph: &mut DomainGraphTransaction<'_>,
) -> Value {
    evaluate(
        graph,
        DomainOperationId::FrameRate,
        vec![Value::Integer(30), Value::Integer(1)],
    )
}

pub(in crate::program::expression::runtime::domain_graph) fn during(
    graph: &mut DomainGraphTransaction<'_>,
) -> Value {
    evaluate(graph, DomainOperationId::During, vec![time(0), time(2)])
}

pub(in crate::program::expression::runtime::domain_graph) fn project_settings(
    graph: &mut DomainGraphTransaction<'_>,
) -> Value {
    evaluate(
        graph,
        DomainOperationId::ProjectSettings,
        vec![Value::Integer(48_000)],
    )
}

pub(in crate::program::expression::runtime::domain_graph) fn sequence_settings(
    graph: &mut DomainGraphTransaction<'_>,
) -> Value {
    let canvas = canvas(graph);
    let rate = frame_rate(graph);
    evaluate(
        graph,
        DomainOperationId::SequenceSettings,
        vec![canvas, rate, Value::Integer(48_000)],
    )
}

pub(in crate::program::expression::runtime::domain_graph) fn track_state(
    graph: &mut DomainGraphTransaction<'_>,
) -> Value {
    let playback = evaluate(graph, DomainOperationId::TrackPlaybackEnabled, vec![]);
    let audio = evaluate(graph, DomainOperationId::TrackAudioAudible, vec![]);
    let isolation = evaluate(graph, DomainOperationId::TrackIsolationNormal, vec![]);
    let editing = evaluate(graph, DomainOperationId::TrackEditingUnlocked, vec![]);
    evaluate(
        graph,
        DomainOperationId::TrackState,
        vec![playback, audio, isolation, editing],
    )
}

pub(in crate::program::expression::runtime::domain_graph) fn generated_source(
    graph: &mut DomainGraphTransaction<'_>,
) -> Value {
    let generator = evaluate(graph, DomainOperationId::GeneratorTransparent, vec![]);
    evaluate(graph, DomainOperationId::SourceGenerated, vec![generator])
}
