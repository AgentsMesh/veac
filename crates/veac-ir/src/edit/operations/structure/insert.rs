use crate::*;

mod sequence;

use self::sequence::insert_sequence;
use super::anchors;
use crate::edit::operations::changed_tree;
use crate::edit::{find_track, operation_error, ChangeSet, MarkChanged};

pub(super) fn apply(
    project: &mut Project,
    edit: &StructureEdit,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    match edit {
        StructureEdit::InsertMaterial {
            material,
            before_id,
            after_id,
        } => insert_material(project, material, before_id, after_id, changed),
        StructureEdit::InsertSequence {
            sequence,
            relations,
            before_id,
            after_id,
        } => insert_sequence(project, sequence, relations, before_id, after_id, changed),
        StructureEdit::InsertTrack {
            sequence_id,
            track,
            before_id,
            after_id,
        } => insert_track(project, sequence_id, track, before_id, after_id, changed),
        StructureEdit::InsertOutput {
            output,
            before_id,
            after_id,
        } => insert_output(project, output, before_id, after_id, changed),
        StructureEdit::InsertMulticamGroup {
            group,
            before_id,
            after_id,
        } => insert_multicam_group(project, group, before_id, after_id, changed),
        _ => unreachable!("insert dispatcher received another structure edit"),
    }
}

fn insert_material(
    project: &mut Project,
    value: &Material,
    before: &Option<MaterialId>,
    after: &Option<MaterialId>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    if project.materials.iter().any(|item| item.id == value.id) {
        return Err(operation_error(
            value.id.as_str(),
            "material ID already exists",
        ));
    }
    let index = anchors::index(
        &project.materials,
        before.as_ref(),
        after.as_ref(),
        |item| &item.id,
        value.id.as_str(),
    )?;
    project.materials.insert(index, value.clone());
    changed.material(value.id.clone());
    changed.project(project.id.clone());
    Ok(())
}

fn insert_track(
    project: &mut Project,
    sequence_id: &SequenceId,
    value: &Track,
    before: &Option<TrackId>,
    after: &Option<TrackId>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    if find_track(project, &value.id).is_some() {
        return Err(operation_error(
            value.id.as_str(),
            "track ID already exists",
        ));
    }
    let sequence = project
        .sequences
        .iter_mut()
        .find(|item| item.id == *sequence_id)
        .ok_or_else(|| operation_error(sequence_id.as_str(), "sequence does not exist"))?;
    let index = anchors::index(
        &sequence.tracks,
        before.as_ref(),
        after.as_ref(),
        |item| &item.id,
        value.id.as_str(),
    )?;
    sequence.tracks.insert(index, value.clone());
    changed_tree::track(value, changed);
    changed.sequence(sequence_id.clone());
    Ok(())
}

fn insert_output(
    project: &mut Project,
    value: &RenderConfig,
    before: &Option<RenderConfigId>,
    after: &Option<RenderConfigId>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    if project
        .render_configs
        .iter()
        .any(|item| item.id == value.id)
    {
        return Err(operation_error(
            value.id.as_str(),
            "output ID already exists",
        ));
    }
    let index = anchors::index(
        &project.render_configs,
        before.as_ref(),
        after.as_ref(),
        |item| &item.id,
        value.id.as_str(),
    )?;
    project.render_configs.insert(index, value.clone());
    changed.output(value.id.clone());
    changed.project(project.id.clone());
    Ok(())
}

fn insert_multicam_group(
    project: &mut Project,
    value: &MulticamGroup,
    before: &Option<MulticamGroupId>,
    after: &Option<MulticamGroupId>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    if project
        .multicam_groups
        .iter()
        .any(|item| item.id == value.id)
    {
        return Err(operation_error(
            value.id.as_str(),
            "multicam group ID already exists",
        ));
    }
    let index = anchors::index(
        &project.multicam_groups,
        before.as_ref(),
        after.as_ref(),
        |item| &item.id,
        value.id.as_str(),
    )?;
    project.multicam_groups.insert(index, value.clone());
    changed.multicam_group(value.id.clone());
    changed.project(project.id.clone());
    Ok(())
}
