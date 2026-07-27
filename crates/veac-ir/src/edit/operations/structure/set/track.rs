use crate::*;

use super::support::{mark_if, track_mut, unlocked_track};
use crate::edit::operations::{apply_refs, relation_refs};
use crate::edit::{operation_error, ChangeSet, MarkChanged};

pub(super) fn apply(
    project: &mut Project,
    edit: &StructureEdit,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let id = match edit {
        StructureEdit::SetTrackState { track_id, state } => {
            return state_edit(project, track_id, *state, changed)
        }
        StructureEdit::SetTrackKind { track_id, kind } => {
            let track = unlocked_track(project, track_id)?;
            mark_if(&mut track.kind, *kind, || changed.track(track_id.clone()));
            track_id
        }
        StructureEdit::SetTrackRouting { track_id, routing } => {
            return routing_edit(project, track_id, routing, changed)
        }
        StructureEdit::SetTrackOrder { track_id, order } => {
            apply_refs::ensure_order_change_unlocked(project, track_id, *order)?;
            let track = unlocked_track(project, track_id)?;
            mark_if(&mut track.order, *order, || changed.track(track_id.clone()));
            track_id
        }
        StructureEdit::SetTrackPlacementMode {
            track_id,
            placement_mode,
        } => {
            let track = unlocked_track(project, track_id)?;
            mark_if(&mut track.placement_mode, *placement_mode, || {
                changed.track(track_id.clone())
            });
            track_id
        }
        _ => unreachable!("track set received another edit"),
    };
    let _ = id;
    Ok(())
}

fn routing_edit(
    project: &mut Project,
    track_id: &TrackId,
    routing: &TrackRouting,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let (sequence_id, old_bus) = project
        .sequences
        .iter()
        .find_map(|sequence| {
            sequence
                .tracks
                .iter()
                .find(|track| track.id == *track_id)
                .map(|track| {
                    let bus = match &track.routing {
                        TrackRouting::AudioBus { bus_id } => Some(bus_id.clone()),
                        _ => None,
                    };
                    (sequence.id.clone(), bus)
                })
        })
        .ok_or_else(|| operation_error(track_id.as_str(), "track does not exist"))?;
    let track = unlocked_track(project, track_id)?;
    if track.routing == *routing {
        return Ok(());
    }
    track.routing = routing.clone();
    changed.track(track_id.clone());
    if let Some(bus_id) = old_bus {
        relation_refs::prune_disappeared_bus(project, &sequence_id, &bus_id, track_id, changed)?;
    }
    Ok(())
}

fn state_edit(
    project: &mut Project,
    id: &TrackId,
    state: TrackState,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let track = track_mut(project, id)?;
    if track.state == state {
        return Ok(());
    }
    let unlock_only = track.state.locked
        && !state.locked
        && track.state.enabled == state.enabled
        && track.state.muted == state.muted
        && track.state.solo == state.solo;
    if track.state.locked && !unlock_only {
        return Err(operation_error(id.as_str(), "track is locked"));
    }
    track.state = state;
    changed.track(id.clone());
    Ok(())
}
