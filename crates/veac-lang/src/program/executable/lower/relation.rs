use crate::program::expression::runtime::domain_graph::{FrozenDomainGraph, FrozenEntity};
use crate::program::expression::Value;
use crate::program::{DomainOperationId as Op, DomainType};
use veac_ir::{
    BusId, MatteRelationParameters, Relation, RelationEndpoint, RelationKind,
    SidechainRelationParameters, TrackMatteMode,
};

use super::error::ExecutableLowerError;
use super::{apply, id, time, value};

mod transition;

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    entity: FrozenEntity<'_>,
    sequence_id: &veac_ir::SequenceId,
    timebase: u32,
) -> Result<Relation, ExecutableLowerError> {
    if entity.domain_type() != Some(DomainType::Relation) {
        return Err(malformed());
    }
    let (operation, values) = entity.constructor().ok_or_else(malformed)?;
    let kind = match (operation, values) {
        (Op::RelationTransition, [_, from, to, transition]) => RelationKind::Transition {
            from: item(graph, from)?,
            to: item(graph, to)?,
            transition: transition::lower(graph, transition, timebase)?,
        },
        (Op::RelationMatteItem, [_, producer, consumer, mode, invert]) => RelationKind::Matte {
            producer: item(graph, producer)?,
            consumer: item(graph, consumer)?,
            parameters: matte(graph, mode, invert)?,
        },
        (Op::RelationMatteApply, [_, producer, consumer, mode, invert]) => RelationKind::Matte {
            producer: item(graph, producer)?,
            consumer: RelationEndpoint::apply(apply::apply_ref(graph, consumer)?),
            parameters: matte(graph, mode, invert)?,
        },
        (Op::RelationSidechainTrack, [_, source, target, settings]) => RelationKind::Sidechain {
            key: RelationEndpoint::track(apply::track_ref(graph, source)?),
            target: item(graph, target)?,
            parameters: sidechain(graph, settings, timebase)?,
        },
        (Op::RelationSidechainBus, [_, source, target, settings]) => RelationKind::Sidechain {
            key: RelationEndpoint::bus(bus(graph, source)?),
            target: item(graph, target)?,
            parameters: sidechain(graph, settings, timebase)?,
        },
        (Op::RelationGroup, [_, members]) => RelationKind::Group {
            members: items(graph, members)?,
        },
        (Op::RelationAvLink, [_, video, audio]) => RelationKind::AvLink {
            video: item(graph, video)?,
            audio: items(graph, audio)?,
        },
        _ => return Err(malformed()),
    };
    Ok(Relation {
        id: id::relation(&entity.logical_path().ok_or_else(malformed)?),
        sequence_id: sequence_id.clone(),
        kind,
    })
}

fn item(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<RelationEndpoint, ExecutableLowerError> {
    Ok(RelationEndpoint::item(apply::item_ref(graph, source)?))
}

fn items(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Vec<RelationEndpoint>, ExecutableLowerError> {
    value::list(Some(source))?
        .iter()
        .map(|source| item(graph, source))
        .collect()
}

fn matte(
    graph: &FrozenDomainGraph,
    mode: &Value,
    invert: &Value,
) -> Result<MatteRelationParameters, ExecutableLowerError> {
    let mode = match value::description(graph, mode)? {
        (Op::MatteAlpha, []) => TrackMatteMode::Alpha,
        (Op::MatteLuma, []) => TrackMatteMode::Luma,
        _ => return Err(malformed()),
    };
    Ok(MatteRelationParameters {
        mode,
        invert: value::boolean(Some(invert))?,
    })
}

fn sidechain(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
) -> Result<SidechainRelationParameters, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::SidechainSettings)?;
    let [threshold, ratio, attack, release, active] = values else {
        return Err(malformed());
    };
    let active_range = match value::description(graph, active)? {
        (Op::SidechainWindowFull, []) => None,
        (Op::SidechainWindowDuring, [range]) => Some(time::range(graph, range, timebase)?),
        _ => return Err(malformed()),
    };
    Ok(SidechainRelationParameters {
        threshold_db: value::finite(Some(threshold))?,
        ratio: value::finite(Some(ratio))?,
        attack_ms: value::finite(Some(attack))?,
        release_ms: value::finite(Some(release))?,
        active_range,
    })
}

fn bus(graph: &FrozenDomainGraph, source: &Value) -> Result<BusId, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::AudioBus)?;
    BusId::new(format!("bus_{}", value::identifier(values.first())?)).map_err(|_| malformed())
}

fn malformed() -> ExecutableLowerError {
    value::graph("an executable relation has invalid typed topology")
}
