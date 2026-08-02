use std::collections::{BTreeMap, BTreeSet};

use crate::edit::{operation_error, ChangeSet, MarkChanged, RelationFragmentId};
use crate::*;

use super::split::SplitPlan;

pub(super) fn validate(
    project: &Project,
    plans: &[SplitPlan],
    fragments: &[RelationFragmentId],
) -> Result<BTreeMap<RelationId, RelationId>, Diagnostic> {
    let expected: BTreeSet<_> = plans
        .iter()
        .filter(|plan| plan.right.is_some())
        .map(|plan| plan.relation.id.clone())
        .collect();
    let mut mapping = BTreeMap::new();
    let mut right_ids = BTreeSet::new();
    for fragment in fragments {
        if mapping
            .insert(
                fragment.relation_id.clone(),
                fragment.right_relation_id.clone(),
            )
            .is_some()
            || !right_ids.insert(fragment.right_relation_id.clone())
        {
            return Err(operation_error(
                "relation_fragments",
                "duplicate relation fragment id",
            ));
        }
        if project
            .relations
            .iter()
            .any(|relation| relation.id == fragment.right_relation_id)
        {
            return Err(operation_error(
                fragment.right_relation_id.as_str(),
                "right relation id already exists",
            ));
        }
    }
    if mapping.keys().cloned().collect::<BTreeSet<_>>() != expected {
        return Err(operation_error(
            "relation_fragments",
            "relation fragment mapping must exactly cover split target relations",
        ));
    }
    Ok(mapping)
}

pub(super) fn apply(
    project: &mut Project,
    right_item_id: &ItemId,
    plans: Vec<SplitPlan>,
    mapping: &BTreeMap<RelationId, RelationId>,
    changed: &mut ChangeSet,
) {
    let mut clones = Vec::new();
    for plan in plans {
        let id = plan.relation.id.clone();
        if let Some(right_range) = plan.right {
            let mut relation = plan.relation.clone();
            relation.id = mapping[&id].clone();
            replace_target(&mut relation.kind, right_item_id, right_range);
            changed.relation(relation.id.clone());
            clones.push(relation);
        }
        match plan.left {
            None => {
                project.relations.retain(|relation| relation.id != id);
                changed.relation(id);
            }
            Some(Some(value)) => {
                let relation = project
                    .relations
                    .iter_mut()
                    .find(|relation| relation.id == id)
                    .expect("split plan relation");
                set_active_range(&mut relation.kind, Some(value));
                changed.relation(id);
            }
            Some(None) => {}
        }
    }
    project.relations.extend(clones);
}

fn replace_target(kind: &mut RelationKind, item_id: &ItemId, range: Option<TimeRange>) {
    match kind {
        RelationKind::Matte { consumer, .. } => *consumer = RelationEndpoint::item(item_id.clone()),
        RelationKind::Sidechain {
            target, parameters, ..
        } => {
            *target = RelationEndpoint::item(item_id.clone());
            parameters.active_range = range;
        }
        _ => unreachable!("split plan contains unsupported relation"),
    }
}

fn set_active_range(kind: &mut RelationKind, range: Option<TimeRange>) {
    if let RelationKind::Sidechain { parameters, .. } = kind {
        parameters.active_range = range;
    }
}
