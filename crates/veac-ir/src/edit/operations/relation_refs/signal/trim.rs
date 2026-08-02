use std::collections::BTreeSet;

use crate::edit::{operation_error, ChangeSet, MarkChanged};
use crate::*;

#[cfg(test)]
#[path = "trim/tests.rs"]
mod tests;

use super::super::lock::ensure_relation_unlocked_except;
use super::range;

pub(super) fn apply(
    project: &mut Project,
    sequence_id: &SequenceId,
    item_id: &ItemId,
    offset: RationalTime,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let duration = item_duration(project, sequence_id, item_id)?;
    let window = TimeRange::new(offset, duration)
        .map_err(|_| operation_error(item_id.as_str(), "trimmed item range is invalid"))?;
    let plans: Vec<_> = project
        .relations
        .iter()
        .filter(|relation| relation.sequence_id == *sequence_id)
        .filter_map(|relation| plan(relation, item_id, &window, offset))
        .collect();
    let skipped = BTreeSet::from([item_id.clone()]);
    for plan in &plans {
        ensure_relation_unlocked_except(project, &plan.relation, &skipped, &BTreeSet::new())?;
    }
    for plan in plans {
        let id = plan.relation.id.clone();
        if let Some(active_range) = plan.active_range {
            let relation = project
                .relations
                .iter_mut()
                .find(|relation| relation.id == id)
                .expect("trim plan relation");
            let RelationKind::Sidechain { parameters, .. } = &mut relation.kind else {
                unreachable!("trim plan contains sidechain relation")
            };
            parameters.active_range = Some(active_range);
        } else {
            project.relations.retain(|relation| relation.id != id);
        }
        changed.relation(id);
    }
    Ok(())
}

struct TrimPlan {
    relation: Relation,
    active_range: Option<TimeRange>,
}

fn plan(
    relation: &Relation,
    item_id: &ItemId,
    window: &TimeRange,
    offset: RationalTime,
) -> Option<TrimPlan> {
    let RelationKind::Sidechain {
        target, parameters, ..
    } = &relation.kind
    else {
        return None;
    };
    if target.item_id() != Some(item_id) {
        return None;
    }
    let active = parameters.active_range.as_ref()?;
    let next = range::intersect(active, window).map(|value| range::rebase(value, offset));
    if next.as_ref() == Some(active) {
        return None;
    }
    Some(TrimPlan {
        relation: relation.clone(),
        active_range: next,
    })
}

fn item_duration(
    project: &Project,
    sequence_id: &SequenceId,
    item_id: &ItemId,
) -> Result<RationalTime, Diagnostic> {
    project
        .sequences
        .iter()
        .find(|sequence| sequence.id == *sequence_id)
        .and_then(|sequence| {
            sequence
                .tracks
                .iter()
                .flat_map(|track| &track.clips)
                .find(|clip| clip.id == *item_id)
        })
        .map(|clip| clip.record_range.duration)
        .ok_or_else(|| operation_error(item_id.as_str(), "trimmed item does not exist"))
}
