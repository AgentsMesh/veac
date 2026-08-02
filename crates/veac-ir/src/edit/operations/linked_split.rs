use crate::edit::operations::time_math::add;
use crate::edit::operations::timing::split_one;
use crate::edit::{operation_error, ChangeSet, MarkChanged};
use crate::{Diagnostic, EditOperation, LinkedSplitFragment, Project, RationalTime, RelationId};

mod support;

use support::{
    av_members, ensure_isolated, ensure_relation_id_available, find_clip, fragment_map,
    remap_av_link,
};

pub(super) fn apply(
    project: &mut Project,
    operation: &EditOperation,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let EditOperation::SplitLinked {
        link_relation_id,
        offset,
        fragments,
        right_link_relation_id,
    } = operation
    else {
        unreachable!("linked split dispatcher received another operation")
    };
    split_linked(
        project,
        link_relation_id,
        *offset,
        fragments,
        right_link_relation_id,
        changed,
    )
}

fn split_linked(
    project: &mut Project,
    link_relation_id: &RelationId,
    offset: RationalTime,
    fragments: &[LinkedSplitFragment],
    right_link_relation_id: &RelationId,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let left_index = project
        .relations
        .iter()
        .position(|relation| &relation.id == link_relation_id)
        .ok_or_else(|| operation_error("EDIT_LINK_NOT_FOUND", "AV-link relation does not exist"))?;
    let link = project.relations[left_index].clone();
    let members = av_members(&link.kind)?;
    ensure_relation_id_available(project, link_relation_id, right_link_relation_id)?;
    ensure_isolated(project, &link.sequence_id, &members)?;
    let by_source = fragment_map(fragments, &members)?;
    let first = find_clip(project, &members[0])?;
    let at = add(first.record_range.start, offset, link_relation_id.as_str())?;
    project.relations.remove(left_index);

    for member in &members {
        let fragment = by_source[member];
        split_one(
            project,
            member,
            at,
            &fragment.right_clip_id,
            &fragment.relation_fragments,
            changed,
        )?;
    }

    ensure_relation_id_available(project, link_relation_id, right_link_relation_id)?;
    let mut right = link.clone();
    right.id = right_link_relation_id.clone();
    remap_av_link(&mut right.kind, &by_source)?;
    project.relations.insert(left_index, link);
    project.relations.insert(left_index + 1, right);
    changed.relation(right_link_relation_id.clone());
    changed.project(project.id.clone());
    Ok(())
}
