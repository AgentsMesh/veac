use crate::program::expression::runtime::domain_graph::{FrozenDomainGraph, FrozenEntity};
use crate::program::expression::Value;
use crate::program::{DomainOperationId as Op, DomainType};
use veac_ir::{
    ClipSource, MulticamAngle, MulticamAngleId, MulticamGroup, MulticamSwitch, MulticamSync,
    MulticamSyncBasis,
};

use super::error::ExecutableLowerError;
use super::{id, time, value};

pub(super) fn group(
    graph: &FrozenDomainGraph,
    entity: FrozenEntity<'_>,
    timebase: u32,
) -> Result<MulticamGroup, ExecutableLowerError> {
    let values = value::entity_operands(entity, DomainType::MulticamGroup, Op::MulticamGroup)?;
    if values.len() != 3 {
        return Err(malformed());
    }
    let path = entity.logical_path().ok_or_else(malformed)?;
    let mut angles = value::list(values.get(2))?
        .iter()
        .map(|source| angle_value(graph, source, timebase, &path))
        .collect::<Result<Vec<_>, _>>()?;
    angles.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    Ok(MulticamGroup {
        id: id::multicam(&path),
        sync: sync(graph, &values[1], &path)?,
        angles,
    })
}

pub(super) fn source(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
) -> Result<ClipSource, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::SourceMulticam)?;
    let [group, switches] = values else {
        return Err(malformed());
    };
    let Value::Domain(group) = group else {
        return Err(malformed());
    };
    let path = graph.logical_key(group).ok_or_else(malformed)?;
    Ok(ClipSource::Multicam {
        group_id: id::multicam(&path),
        switches: value::list(Some(switches))?
            .iter()
            .map(|source| switch(graph, source, timebase, &path))
            .collect::<Result<_, _>>()?,
    })
}

fn angle_value(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    group: &[&str],
) -> Result<MulticamAngle, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::MulticamAngle)?;
    let [key, resource, offset] = values else {
        return Err(malformed());
    };
    let mut path = group.to_vec();
    path.push(value::identifier(Some(key))?);
    let Value::Domain(resource) = resource else {
        return Err(malformed());
    };
    Ok(MulticamAngle {
        id: id::angle(&path),
        material_id: id::material(&graph.logical_key(resource).ok_or_else(malformed)?),
        source_offset: time::coordinate(Some(offset), timebase)?,
    })
}

fn sync(
    graph: &FrozenDomainGraph,
    source: &Value,
    group: &[&str],
) -> Result<MulticamSync, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::MulticamSync)?;
    let [basis, reference] = values else {
        return Err(malformed());
    };
    Ok(MulticamSync {
        basis: match value::description(graph, basis)? {
            (Op::MulticamSyncTimecode, []) => MulticamSyncBasis::Timecode,
            (Op::MulticamSyncAudio, []) => MulticamSyncBasis::Audio,
            (Op::MulticamSyncManual, []) => MulticamSyncBasis::Manual,
            _ => return Err(malformed()),
        },
        reference_angle_id: angle_ref(graph, reference, group)?,
    })
}

fn switch(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    group: &[&str],
) -> Result<MulticamSwitch, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::MulticamSwitch)?;
    let [angle, range] = values else {
        return Err(malformed());
    };
    Ok(MulticamSwitch {
        angle_id: angle_ref(graph, angle, group)?,
        range: time::range(graph, range, timebase)?,
    })
}

fn angle_ref(
    graph: &FrozenDomainGraph,
    source: &Value,
    group: &[&str],
) -> Result<MulticamAngleId, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::MulticamAngle)?;
    let mut path = group.to_vec();
    path.push(value::identifier(values.first())?);
    Ok(id::angle(&path))
}

fn malformed() -> ExecutableLowerError {
    value::graph("an executable multicam graph has invalid typed topology")
}
