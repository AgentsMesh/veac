use crate::edit::{ensure_clip_unlocked, operation_error, ChangeSet, MarkChanged};
use crate::*;

#[cfg(test)]
#[path = "transition/tests.rs"]
mod tests;

pub(super) fn set(
    project: &mut Project,
    clip_id: &ItemId,
    transition: &Option<Transition>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    ensure_clip_unlocked(project, clip_id)?;
    let (sequence_id, next_id) = locate(project, clip_id)?;
    let existing = project.relations.iter().position(|relation| {
        relation.sequence_id == sequence_id
            && matches!(
                &relation.kind,
                RelationKind::Transition { from, .. }
                    if from.item_id() == Some(clip_id)
            )
    });
    if existing.is_none() && transition.is_none() {
        return Ok(());
    }
    let target_id = existing
        .and_then(|index| match &project.relations[index].kind {
            RelationKind::Transition { to, .. } => to.item_id().cloned(),
            _ => None,
        })
        .or(next_id)
        .ok_or_else(|| {
            operation_error(clip_id.as_str(), "transition source has no following item")
        })?;
    ensure_clip_unlocked(project, &target_id)?;
    match (existing, transition) {
        (Some(index), Some(value)) => {
            let relation_id = project.relations[index].id.clone();
            let RelationKind::Transition { transition, .. } = &mut project.relations[index].kind
            else {
                unreachable!()
            };
            if transition != value {
                *transition = value.clone();
                changed.item(clip_id.clone());
                changed.item(target_id);
                changed.relation(relation_id);
            }
        }
        (Some(index), None) => {
            let relation = project.relations.remove(index);
            changed.item(clip_id.clone());
            changed.item(target_id);
            changed.relation(relation.id);
        }
        (None, Some(value)) => {
            let relation_id = relation_id(project, clip_id)?;
            project.relations.push(Relation {
                id: relation_id.clone(),
                sequence_id,
                kind: RelationKind::Transition {
                    from: RelationEndpoint::item(clip_id.clone()),
                    to: RelationEndpoint::item(target_id.clone()),
                    transition: value.clone(),
                },
            });
            changed.item(clip_id.clone());
            changed.item(target_id);
            changed.relation(relation_id);
        }
        (None, None) => {}
    }
    Ok(())
}

fn locate(project: &Project, clip_id: &ItemId) -> Result<(SequenceId, Option<ItemId>), Diagnostic> {
    project
        .sequences
        .iter()
        .find_map(|sequence| {
            sequence.tracks.iter().find_map(|track| {
                track
                    .clips
                    .iter()
                    .position(|clip| clip.id == *clip_id)
                    .map(|position| {
                        (
                            sequence.id.clone(),
                            track.clips.get(position + 1).map(|clip| clip.id.clone()),
                        )
                    })
            })
        })
        .ok_or_else(|| operation_error(clip_id.as_str(), "clip does not exist"))
}

fn relation_id(project: &Project, clip_id: &ItemId) -> Result<RelationId, Diagnostic> {
    let id = RelationId::new(format!("rel_transition_{}", clip_id.as_str()))
        .map_err(|_| operation_error(clip_id.as_str(), "cannot derive transition relation id"))?;
    if project.relations.iter().all(|relation| relation.id != id) {
        Ok(id)
    } else {
        Err(operation_error(
            clip_id.as_str(),
            "transition relation ID is already in use",
        ))
    }
}
