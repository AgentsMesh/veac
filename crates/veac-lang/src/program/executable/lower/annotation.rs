use crate::program::expression::runtime::domain_graph::{FrozenDomainGraph, FrozenEntity};
use crate::program::expression::Value;
use crate::program::{DomainOperationId as Op, DomainType};
use veac_ir::{Annotation, AnnotationProvenance, AnnotationSpan, AnnotationTarget};

use super::error::ExecutableLowerError;
use super::{id, time, value};

mod payload;

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    entity: FrozenEntity<'_>,
    timebase: u32,
) -> Result<Annotation, ExecutableLowerError> {
    let values = value::entity_operands(entity, DomainType::Annotation, Op::Annotation)?;
    if values.len() != 5 {
        return Err(malformed());
    }
    let target = target(graph, &values[1])?;
    Ok(Annotation {
        id: id::annotation(&entity.logical_path().ok_or_else(malformed)?),
        target: target.clone(),
        span: span(
            graph,
            &values[2],
            timebase,
            matches!(target, AnnotationTarget::Material { .. }),
        )?,
        payload: payload::lower(graph, &values[3])?,
        provenance: provenance(graph, &values[4])?,
    })
}

fn target(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<AnnotationTarget, ExecutableLowerError> {
    let reference = |value: &Value| -> Result<Vec<&str>, ExecutableLowerError> {
        let Value::Domain(handle) = value else {
            return Err(malformed());
        };
        graph.logical_key(handle).ok_or_else(malformed)
    };
    match value::description(graph, source)? {
        (Op::AnnotationTargetProject, []) => Ok(AnnotationTarget::Project),
        (Op::AnnotationTargetSequence, [value]) => Ok(AnnotationTarget::Sequence {
            sequence_id: id::sequence(&reference(value)?),
        }),
        (Op::AnnotationTargetLayer, [value]) => Ok(AnnotationTarget::Track {
            track_id: id::track(&reference(value)?),
        }),
        (Op::AnnotationTargetItem, [value]) => Ok(AnnotationTarget::Clip {
            clip_id: id::item(&reference(value)?),
        }),
        (Op::AnnotationTargetResource, [value]) => Ok(AnnotationTarget::Material {
            material_id: id::material(&reference(value)?),
        }),
        (Op::AnnotationTargetMulticam, [value]) => Ok(AnnotationTarget::MulticamGroup {
            group_id: id::multicam(&reference(value)?),
        }),
        _ => Err(malformed()),
    }
}

fn span(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    intrinsic: bool,
) -> Result<AnnotationSpan, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::AnnotationUntimed, []) => Ok(AnnotationSpan::Untimed),
        (Op::AnnotationPoint, [at]) => Ok(AnnotationSpan::Point {
            at: coordinate(Some(at), timebase, intrinsic)?,
        }),
        (Op::AnnotationRange, [range]) if intrinsic => {
            let values = value::description_operands(graph, range, Op::During)?;
            Ok(AnnotationSpan::Range {
                range: veac_ir::TimeRange::new(
                    time::intrinsic(values.first())?,
                    time::intrinsic(values.get(1))?,
                )
                .map_err(|_| malformed())?,
            })
        }
        (Op::AnnotationRange, [range]) => Ok(AnnotationSpan::Range {
            range: time::range(graph, range, timebase)?,
        }),
        _ => Err(malformed()),
    }
}

fn coordinate(
    value: Option<&Value>,
    timebase: u32,
    intrinsic: bool,
) -> Result<veac_ir::RationalTime, ExecutableLowerError> {
    if intrinsic {
        time::intrinsic(value)
    } else {
        time::coordinate(value, timebase)
    }
}

fn provenance(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<AnnotationProvenance>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::AnnotationProvenanceNone, []) => Ok(None),
        (Op::AnnotationProvenancePresent, [source]) => {
            let values = value::description_operands(graph, source, Op::AnnotationProvenance)?;
            let [producer, request, response] = values else {
                return Err(malformed());
            };
            Ok(Some(AnnotationProvenance {
                producer: value::text(Some(producer))?.to_owned(),
                request_sha256: digest(graph, request)?,
                response_sha256: digest(graph, response)?,
            }))
        }
        _ => Err(malformed()),
    }
}

fn digest(graph: &FrozenDomainGraph, source: &Value) -> Result<String, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::Sha256)?;
    Ok(value::text(values.first())?.to_owned())
}

fn malformed() -> ExecutableLowerError {
    value::graph("an executable annotation has invalid typed topology")
}
