use crate::*;

use super::references::{
    material_is_referenced, multicam_group_is_referenced, multicam_group_is_used_on_locked_track,
    sequence_is_referenced,
};
use crate::edit::operations::relation_refs;
use crate::edit::operations::{apply_refs, changed_tree};
use crate::edit::{operation_error, ChangeSet, MarkChanged};

pub(super) fn apply(
    project: &mut Project,
    edit: &StructureEdit,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    match edit {
        StructureEdit::RemoveMaterial { material_id } => material(project, material_id, changed),
        StructureEdit::RemoveSequence { sequence_id } => sequence(project, sequence_id, changed),
        StructureEdit::RemoveTrack { track_id } => track(project, track_id, changed),
        StructureEdit::RemoveOutput { output_id } => output(project, output_id, changed),
        StructureEdit::RemoveMulticamGroup { group_id } => {
            multicam_group(project, group_id, changed)
        }
        _ => unreachable!("remove dispatcher received another structure edit"),
    }
}

fn material(
    project: &mut Project,
    id: &MaterialId,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    if material_is_referenced(project, id) {
        return Err(operation_error(id.as_str(), "material is still referenced"));
    }
    let index = project
        .materials
        .iter()
        .position(|value| value.id == *id)
        .ok_or_else(|| operation_error(id.as_str(), "material does not exist"))?;
    project.materials.remove(index);
    changed.material(id.clone());
    changed.project(project.id.clone());
    Ok(())
}

fn sequence(
    project: &mut Project,
    id: &SequenceId,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    if sequence_is_referenced(project, id) {
        return Err(operation_error(id.as_str(), "sequence is still referenced"));
    }
    let index = project
        .sequences
        .iter()
        .position(|value| value.id == *id)
        .ok_or_else(|| operation_error(id.as_str(), "sequence does not exist"))?;
    if project.sequences[index]
        .tracks
        .iter()
        .any(|track| track.state.locked)
    {
        return Err(operation_error(
            id.as_str(),
            "sequence contains a locked track",
        ));
    }
    relation_refs::remove_sequence(project, id, changed);
    let removed = project.sequences.remove(index);
    changed_tree::sequence(&removed, changed);
    changed.project(project.id.clone());
    Ok(())
}

fn track(project: &mut Project, id: &TrackId, changed: &mut ChangeSet) -> Result<(), Diagnostic> {
    apply_refs::ensure_track_removable(project, id)?;
    let (sequence_index, track_index) = project
        .sequences
        .iter()
        .enumerate()
        .find_map(|(sequence_index, sequence)| {
            sequence
                .tracks
                .iter()
                .position(|track| track.id == *id)
                .map(|track_index| (sequence_index, track_index))
        })
        .ok_or_else(|| operation_error(id.as_str(), "track does not exist"))?;
    if project.sequences[sequence_index].tracks[track_index]
        .state
        .locked
    {
        return Err(operation_error(id.as_str(), "track is locked"));
    }
    let sequence_id = project.sequences[sequence_index].id.clone();
    relation_refs::remove_track(project, &sequence_id, id, changed)?;
    let sequence = &mut project.sequences[sequence_index];
    let removed = sequence.tracks.remove(track_index);
    changed_tree::track(&removed, changed);
    changed.sequence(sequence.id.clone());
    Ok(())
}

fn output(
    project: &mut Project,
    id: &RenderConfigId,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let index = project
        .render_configs
        .iter()
        .position(|value| value.id == *id)
        .ok_or_else(|| operation_error(id.as_str(), "output does not exist"))?;
    project.render_configs.remove(index);
    changed.output(id.clone());
    changed.project(project.id.clone());
    Ok(())
}

fn multicam_group(
    project: &mut Project,
    id: &MulticamGroupId,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    if multicam_group_is_used_on_locked_track(project, id) {
        return Err(operation_error(
            id.as_str(),
            "multicam group is used by a clip on a locked track",
        ));
    }
    if multicam_group_is_referenced(project, id) {
        return Err(operation_error(
            id.as_str(),
            "multicam group is still referenced",
        ));
    }
    let index = project
        .multicam_groups
        .iter()
        .position(|value| value.id == *id)
        .ok_or_else(|| operation_error(id.as_str(), "multicam group does not exist"))?;
    project.multicam_groups.remove(index);
    changed.multicam_group(id.clone());
    changed.project(project.id.clone());
    Ok(())
}
