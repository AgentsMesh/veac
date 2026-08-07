use crate::program::expression::runtime::domain_graph::{FrozenDomainGraph, FrozenEntity};
use crate::program::{DomainOperationId as Op, DomainType};
use veac_ir::{BusId, PlacementMode, TrackKind, TrackRouting, TrackState};

use super::super::error::ExecutableLowerError;
use super::super::value;

pub(in crate::program::executable::lower) struct LayerSettings {
    pub(in crate::program::executable::lower) kind: TrackKind,
    pub(in crate::program::executable::lower) order: i32,
    pub(in crate::program::executable::lower) placement: PlacementMode,
    pub(in crate::program::executable::lower) state: TrackState,
    pub(in crate::program::executable::lower) routing: TrackRouting,
}

pub(in crate::program::executable::lower) fn layer(
    graph: &FrozenDomainGraph,
    entity: FrozenEntity<'_>,
) -> Result<LayerSettings, ExecutableLowerError> {
    let (operation, operands) = entity.constructor().ok_or_else(malformed)?;
    if entity.domain_type() != Some(DomainType::Layer) || operands.len() != 5 {
        return Err(malformed());
    }
    let kind = match operation {
        Op::VideoLayer => TrackKind::Video,
        Op::AudioLayer => TrackKind::Audio,
        Op::VisualLayer => TrackKind::Visual,
        Op::CaptionLayer => TrackKind::Caption,
        _ => return Err(malformed()),
    };
    Ok(LayerSettings {
        kind,
        order: i32::try_from(value::integer(operands.get(1))?).map_err(|_| malformed())?,
        placement: placement(graph, operands.get(2).ok_or_else(malformed)?)?,
        state: state(graph, operands.get(3).ok_or_else(malformed)?)?,
        routing: routing(graph, operands.get(4).ok_or_else(malformed)?)?,
    })
}

fn placement(
    graph: &FrozenDomainGraph,
    value: &crate::program::expression::Value,
) -> Result<PlacementMode, ExecutableLowerError> {
    match value::description(graph, value)? {
        (Op::PlacementMagnetic, []) => Ok(PlacementMode::Magnetic),
        (Op::PlacementFree, []) => Ok(PlacementMode::Free),
        _ => Err(malformed()),
    }
}

fn state(
    graph: &FrozenDomainGraph,
    value: &crate::program::expression::Value,
) -> Result<TrackState, ExecutableLowerError> {
    let operands = value::description_operands(graph, value, Op::TrackState)?;
    if operands.len() != 4 {
        return Err(malformed());
    }
    Ok(TrackState {
        enabled: flag(
            graph,
            &operands[0],
            Op::TrackPlaybackEnabled,
            Op::TrackPlaybackDisabled,
        )?,
        muted: flag(
            graph,
            &operands[1],
            Op::TrackAudioMuted,
            Op::TrackAudioAudible,
        )?,
        solo: flag(
            graph,
            &operands[2],
            Op::TrackIsolationSolo,
            Op::TrackIsolationNormal,
        )?,
        locked: flag(
            graph,
            &operands[3],
            Op::TrackEditingLocked,
            Op::TrackEditingUnlocked,
        )?,
    })
}

fn routing(
    graph: &FrozenDomainGraph,
    value: &crate::program::expression::Value,
) -> Result<TrackRouting, ExecutableLowerError> {
    match value::description(graph, value)? {
        (Op::TrackRoutingDefault, []) => Ok(TrackRouting::Default),
        (Op::TrackRoutingBus, [bus]) => {
            let values = value::description_operands(graph, bus, Op::AudioBus)?;
            let key = value::identifier(values.first())?;
            let id = BusId::new(format!("bus_{key}")).map_err(|_| malformed())?;
            Ok(TrackRouting::AudioBus { bus_id: id })
        }
        _ => Err(malformed()),
    }
}

fn flag(
    graph: &FrozenDomainGraph,
    value: &crate::program::expression::Value,
    yes: Op,
    no: Op,
) -> Result<bool, ExecutableLowerError> {
    match value::description(graph, value)? {
        (operation, []) if operation == yes => Ok(true),
        (operation, []) if operation == no => Ok(false),
        _ => Err(malformed()),
    }
}

fn malformed() -> ExecutableLowerError {
    value::graph("the executable layer has invalid typed settings")
}
