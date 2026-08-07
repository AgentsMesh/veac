use crate::program::expression::runtime::domain_graph::{FrozenDomainGraph, FrozenEntity};
use crate::program::{DomainOperationId as Op, DomainType};
use veac_ir::{Relation, Sequence, Track};

use super::error::ExecutableLowerError;
use super::{apply, clip, id, relation, settings, value};

pub(super) struct LoweredSequence {
    pub(super) sequence: Sequence,
    pub(super) relations: Vec<Relation>,
}

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    entity: FrozenEntity<'_>,
    timebase: u32,
) -> Result<LoweredSequence, ExecutableLowerError> {
    let operands = value::entity_operands(entity, DomainType::Sequence, Op::Sequence)?;
    if operands.len() != 3 {
        return Err(malformed());
    }
    let mut tracks = Vec::new();
    let mut applies = Vec::new();
    let mut relations = Vec::new();
    let path = entity.logical_path().ok_or_else(malformed)?;
    let sequence_id = id::sequence(&path);
    for child in entity.children() {
        match child.domain_type() {
            Some(DomainType::Layer) => tracks.push(track(graph, child, timebase)?),
            Some(DomainType::Apply) => applies.push(apply::lower(graph, child, timebase)?),
            Some(DomainType::Relation) => {
                relations.push(relation::lower(graph, child, &sequence_id, timebase)?)
            }
            _ => return Err(malformed()),
        }
    }
    Ok(LoweredSequence {
        sequence: Sequence {
            id: sequence_id,
            name: value::text(operands.get(1))?.to_owned(),
            settings: settings::sequence(graph, entity)?,
            tracks,
            applies,
            authorship: Some(super::metadata::sequence(entity)?),
        },
        relations,
    })
}

fn track(
    graph: &FrozenDomainGraph,
    entity: FrozenEntity<'_>,
    timebase: u32,
) -> Result<Track, ExecutableLowerError> {
    let settings = settings::layer(graph, entity)?;
    let mut clips = entity
        .children()
        .map(|item| clip::lower(graph, item, timebase, settings.order, settings.kind))
        .collect::<Result<Vec<_>, _>>()?;
    clips.sort_by_key(|clip| clip.record_range.start.value);
    Ok(Track {
        id: id::track(&entity.logical_path().ok_or_else(malformed)?),
        kind: settings.kind,
        order: settings.order,
        placement_mode: settings.placement,
        state: settings.state,
        routing: settings.routing,
        clips,
    })
}

fn malformed() -> ExecutableLowerError {
    value::graph("an executable sequence has invalid typed layer topology")
}
