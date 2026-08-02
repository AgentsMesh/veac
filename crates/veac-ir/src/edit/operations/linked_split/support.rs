use std::collections::{BTreeMap, BTreeSet};

use crate::edit::operation_error;
use crate::{
    Diagnostic, ItemId, LinkedSplitFragment, Project, RelationEndpoint, RelationGraph, RelationId,
    RelationKind, SequenceId,
};

pub(super) fn av_members(kind: &RelationKind) -> Result<Vec<ItemId>, Diagnostic> {
    let RelationKind::AvLink { video, audio } = kind else {
        return Err(operation_error(
            "EDIT_LINK_NOT_FOUND",
            "relation is not an AV-link",
        ));
    };
    let mut members = vec![item_id(video)?];
    for endpoint in audio {
        members.push(item_id(endpoint)?);
    }
    Ok(members)
}

fn item_id(endpoint: &RelationEndpoint) -> Result<ItemId, Diagnostic> {
    let RelationEndpoint::Item { item_id } = endpoint else {
        return Err(operation_error(
            "EDIT_LINK_INVALID",
            "AV-link endpoints must be clips",
        ));
    };
    Ok(item_id.clone())
}

pub(super) fn fragment_map<'a>(
    fragments: &'a [LinkedSplitFragment],
    members: &[ItemId],
) -> Result<BTreeMap<ItemId, &'a LinkedSplitFragment>, Diagnostic> {
    let mut by_source = BTreeMap::new();
    let mut right_ids = BTreeSet::new();
    for fragment in fragments {
        if by_source
            .insert(fragment.source_clip_id.clone(), fragment)
            .is_some()
            || !right_ids.insert(fragment.right_clip_id.clone())
        {
            return Err(operation_error(
                "EDIT_DUPLICATE_ID",
                "linked split fragment IDs must be unique",
            ));
        }
    }
    let expected = members.iter().cloned().collect::<BTreeSet<_>>();
    if by_source.keys().cloned().collect::<BTreeSet<_>>() != expected {
        return Err(operation_error(
            "EDIT_LINK_MEMBERSHIP_MISMATCH",
            "linked split fragments must exactly match AV-link members",
        ));
    }
    Ok(by_source)
}

pub(super) fn ensure_relation_id_available(
    project: &Project,
    left_id: &RelationId,
    right_id: &RelationId,
) -> Result<(), Diagnostic> {
    if left_id == right_id
        || project
            .relations
            .iter()
            .any(|relation| &relation.id == right_id)
    {
        return Err(operation_error(
            "EDIT_DUPLICATE_ID",
            "right AV-link relation ID already exists",
        ));
    }
    Ok(())
}

pub(super) fn ensure_isolated(
    project: &Project,
    sequence_id: &SequenceId,
    members: &[ItemId],
) -> Result<(), Diagnostic> {
    let graph = RelationGraph::project(project);
    if members
        .iter()
        .any(|member| graph.group_for_item(sequence_id, member).is_some())
    {
        return Err(operation_error(
            "EDIT_RELATIONSHIP_CONFLICT",
            "linked split does not support clips that also belong to a group",
        ));
    }
    Ok(())
}

pub(super) fn find_clip<'a>(
    project: &'a Project,
    clip_id: &ItemId,
) -> Result<&'a crate::Clip, Diagnostic> {
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
        .find(|clip| &clip.id == clip_id)
        .ok_or_else(|| operation_error("EDIT_ITEM_NOT_FOUND", "linked clip does not exist"))
}

pub(super) fn remap_av_link(
    kind: &mut RelationKind,
    fragments: &BTreeMap<ItemId, &LinkedSplitFragment>,
) -> Result<(), Diagnostic> {
    let RelationKind::AvLink { video, audio } = kind else {
        unreachable!("validated relation kind changed during linked split")
    };
    remap_endpoint(video, fragments)?;
    for endpoint in audio {
        remap_endpoint(endpoint, fragments)?;
    }
    Ok(())
}

fn remap_endpoint(
    endpoint: &mut RelationEndpoint,
    fragments: &BTreeMap<ItemId, &LinkedSplitFragment>,
) -> Result<(), Diagnostic> {
    let source_id = item_id(endpoint)?;
    *endpoint = RelationEndpoint::Item {
        item_id: fragments[&source_id].right_clip_id.clone(),
    };
    Ok(())
}
