use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::ClipSource;

use super::super::error::ExecutableLowerError;
use super::super::{generator, id, multicam, time, value};

pub(super) struct LoweredSource {
    pub(super) value: ClipSource,
    pub(super) timed: bool,
}

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    duration: veac_ir::RationalTime,
    scope: &[&str],
) -> Result<LoweredSource, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::SourceMedia, [resource]) => Ok(LoweredSource {
            value: ClipSource::Media {
                material_id: material_ref(graph, resource)?,
            },
            timed: true,
        }),
        (Op::SourceFreezeFrame, [resource, source_at]) => Ok(LoweredSource {
            value: ClipSource::FreezeFrame {
                material_id: material_ref(graph, resource)?,
                source_time: time::coordinate(Some(source_at), timebase)?,
            },
            timed: false,
        }),
        (Op::SourceNestedSequence, [sequence]) => Ok(LoweredSource {
            value: ClipSource::Sequence {
                sequence_id: sequence_ref(graph, sequence)?,
            },
            timed: true,
        }),
        (Op::SourceGenerated, [source]) => Ok(LoweredSource {
            value: ClipSource::Generated {
                generator: generator::lower(graph, source)?,
            },
            timed: false,
        }),
        (Op::SourceText | Op::SourceCaption | Op::SourceCaptionSpeaker, _) => Ok(LoweredSource {
            value: super::text::lower(graph, source, timebase, duration, scope)?,
            timed: false,
        }),
        (Op::SourceMulticam, _) => Ok(LoweredSource {
            value: multicam::source(graph, source, timebase)?,
            timed: false,
        }),
        _ => Err(malformed()),
    }
}

fn material_ref(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<veac_ir::MaterialId, ExecutableLowerError> {
    let Value::Domain(handle) = source else {
        return Err(malformed());
    };
    let path = graph.logical_key(handle).ok_or_else(malformed)?;
    Ok(id::material(&path))
}

fn sequence_ref(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<veac_ir::SequenceId, ExecutableLowerError> {
    let Value::Domain(handle) = source else {
        return Err(malformed());
    };
    let path = graph.logical_key(handle).ok_or_else(malformed)?;
    Ok(id::sequence(&path))
}

fn malformed() -> ExecutableLowerError {
    value::graph("an executable item has invalid typed source topology")
}
