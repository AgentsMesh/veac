use crate::*;

mod support;
mod track;

use self::support::{mark_if, material_index, sequence_mut};
use super::references::material_is_used_on_locked_track;
use crate::edit::{operation_error, ChangeSet, MarkChanged};

pub(super) fn apply(
    project: &mut Project,
    edit: &StructureEdit,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    match edit {
        StructureEdit::SetMaterial {
            material_id,
            material,
        } => set_material(project, material_id, material, changed),
        StructureEdit::RelinkMaterial {
            material_id,
            source,
            identity,
            probe,
        } => relink(project, material_id, source, identity, probe, changed),
        StructureEdit::SetSequenceName { sequence_id, name } => {
            let sequence = sequence_mut(project, sequence_id)?;
            mark_if(&mut sequence.name, name.clone(), || {
                changed.sequence(sequence_id.clone())
            });
            Ok(())
        }
        StructureEdit::SetSequenceSettings {
            sequence_id,
            settings,
        } => {
            let sequence = sequence_mut(project, sequence_id)?;
            mark_if(&mut sequence.settings, settings.clone(), || {
                changed.sequence(sequence_id.clone())
            });
            Ok(())
        }
        StructureEdit::SetTrackState { .. }
        | StructureEdit::SetTrackKind { .. }
        | StructureEdit::SetTrackRouting { .. }
        | StructureEdit::SetTrackOrder { .. }
        | StructureEdit::SetTrackPlacementMode { .. } => track::apply(project, edit, changed),
        StructureEdit::SetOutput { output_id, output } => {
            set_output(project, output_id, output, changed)
        }
        StructureEdit::SetEntrySequence { sequence_id } => set_entry(project, sequence_id, changed),
        _ => unreachable!("set dispatcher received an insert or remove edit"),
    }
}

fn set_material(
    project: &mut Project,
    id: &MaterialId,
    value: &Material,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    if value.id != *id {
        return Err(operation_error(
            id.as_str(),
            "replacement material ID must match",
        ));
    }
    let index = material_index(project, id)?;
    if project.materials[index] == *value {
        return Ok(());
    }
    if material_is_used_on_locked_track(project, id) {
        return Err(operation_error(
            id.as_str(),
            "material is used on a locked track",
        ));
    }
    project.materials[index] = value.clone();
    changed.material(id.clone());
    Ok(())
}

fn relink(
    project: &mut Project,
    id: &MaterialId,
    source: &MaterialSource,
    identity: &Option<MediaIdentity>,
    probe: &Option<Box<MediaProbeSnapshot>>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let index = material_index(project, id)?;
    let material = &project.materials[index];
    if material.source == *source
        && material.identity == *identity
        && material.probe.as_ref() == probe.as_deref()
    {
        return Ok(());
    }
    if material_is_used_on_locked_track(project, id) {
        return Err(operation_error(
            id.as_str(),
            "material is used on a locked track",
        ));
    }
    let material = &mut project.materials[index];
    material.source = source.clone();
    material.identity = identity.clone();
    material.probe = probe.as_deref().cloned();
    changed.material(id.clone());
    Ok(())
}

fn set_output(
    project: &mut Project,
    id: &RenderConfigId,
    output: &RenderConfig,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    if output.id != *id {
        return Err(operation_error(
            id.as_str(),
            "replacement output ID must match",
        ));
    }
    let value = project
        .render_configs
        .iter_mut()
        .find(|value| value.id == *id)
        .ok_or_else(|| operation_error(id.as_str(), "output does not exist"))?;
    mark_if(value, output.clone(), || changed.output(id.clone()));
    Ok(())
}

fn set_entry(
    project: &mut Project,
    id: &SequenceId,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    if !project.sequences.iter().any(|value| value.id == *id) {
        return Err(operation_error(
            id.as_str(),
            "entry sequence does not exist",
        ));
    }
    if project.entry_sequence_id != *id {
        project.entry_sequence_id = id.clone();
        changed.project(project.id.clone());
    }
    Ok(())
}
