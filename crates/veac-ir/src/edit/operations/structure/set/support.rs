use crate::*;

use crate::edit::operation_error;

#[cfg(test)]
mod tests;

pub(super) fn material_index(project: &Project, id: &MaterialId) -> Result<usize, Diagnostic> {
    project
        .materials
        .iter()
        .position(|value| value.id == *id)
        .ok_or_else(|| operation_error(id.as_str(), "material does not exist"))
}

pub(super) fn sequence_mut<'a>(
    project: &'a mut Project,
    id: &SequenceId,
) -> Result<&'a mut Sequence, Diagnostic> {
    project
        .sequences
        .iter_mut()
        .find(|value| value.id == *id)
        .ok_or_else(|| operation_error(id.as_str(), "sequence does not exist"))
}

pub(super) fn track_mut<'a>(
    project: &'a mut Project,
    id: &TrackId,
) -> Result<&'a mut Track, Diagnostic> {
    project
        .sequences
        .iter_mut()
        .flat_map(|sequence| &mut sequence.tracks)
        .find(|value| value.id == *id)
        .ok_or_else(|| operation_error(id.as_str(), "track does not exist"))
}

pub(super) fn unlocked_track<'a>(
    project: &'a mut Project,
    id: &TrackId,
) -> Result<&'a mut Track, Diagnostic> {
    let track = track_mut(project, id)?;
    if track.state.locked {
        Err(operation_error(id.as_str(), "track is locked"))
    } else {
        Ok(track)
    }
}

pub(super) fn mark_if<T: PartialEq>(target: &mut T, value: T, mark: impl FnOnce()) {
    if *target != value {
        *target = value;
        mark();
    }
}
