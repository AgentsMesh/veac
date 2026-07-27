use std::collections::BTreeSet;

use crate::edit::{operation_error, ChangeSet, RelationFragmentId};
use crate::*;

use super::super::lock::ensure_relation_unlocked_except;
use super::{fragment, range};

pub(super) fn apply(
    project: &mut Project,
    sequence_id: &SequenceId,
    item_id: &ItemId,
    right_item_id: &ItemId,
    fragments: &[RelationFragmentId],
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    reject_unsupported(project, sequence_id, item_id)?;
    let windows = range::item_windows(project, sequence_id, item_id, right_item_id)?;
    let plans: Vec<_> = project
        .relations
        .iter()
        .filter(|relation| relation.sequence_id == *sequence_id)
        .filter_map(|relation| plan(relation, item_id, &windows))
        .collect();
    let mapping = fragment::validate(project, &plans, fragments)?;
    let skipped = BTreeSet::from([item_id.clone(), right_item_id.clone()]);
    for plan in &plans {
        ensure_relation_unlocked_except(project, &plan.relation, &skipped, &BTreeSet::new())?;
    }
    fragment::apply(project, right_item_id, plans, &mapping, changed);
    Ok(())
}

pub(super) struct SplitPlan {
    pub(super) relation: Relation,
    pub(super) left: Option<Option<TimeRange>>,
    pub(super) right: Option<Option<TimeRange>>,
}

fn plan(relation: &Relation, item_id: &ItemId, windows: &[TimeRange; 2]) -> Option<SplitPlan> {
    match &relation.kind {
        RelationKind::Matte { consumer, .. } if consumer.item_id() == Some(item_id) => {
            Some(SplitPlan {
                relation: relation.clone(),
                left: Some(None),
                right: Some(None),
            })
        }
        RelationKind::Sidechain {
            target, parameters, ..
        } if target.item_id() == Some(item_id) => {
            let Some(active) = &parameters.active_range else {
                return Some(SplitPlan {
                    relation: relation.clone(),
                    left: Some(None),
                    right: Some(None),
                });
            };
            Some(SplitPlan {
                relation: relation.clone(),
                left: range::intersect(active, &windows[0]).map(Some),
                right: range::intersect(active, &windows[1])
                    .map(|value| Some(range::rebase(value, windows[1].start))),
            })
        }
        _ => None,
    }
}

fn reject_unsupported(
    project: &Project,
    sequence_id: &SequenceId,
    item_id: &ItemId,
) -> Result<(), Diagnostic> {
    let blocked = project.relations.iter().find(|relation| {
        relation.sequence_id == *sequence_id
            && match &relation.kind {
                RelationKind::Matte { producer, .. } => producer.item_id() == Some(item_id),
                RelationKind::Group { members } => members
                    .iter()
                    .any(|endpoint| endpoint.item_id() == Some(item_id)),
                RelationKind::AvLink { video, audio } => {
                    video.item_id() == Some(item_id)
                        || audio
                            .iter()
                            .any(|endpoint| endpoint.item_id() == Some(item_id))
                }
                _ => false,
            }
    });
    if let Some(relation) = blocked {
        Err(operation_error(
            relation.id.as_str(),
            "relation producer or linked member requires a dedicated split primitive",
        ))
    } else {
        Ok(())
    }
}
