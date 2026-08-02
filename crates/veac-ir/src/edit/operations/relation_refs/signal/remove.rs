use std::collections::BTreeSet;

use crate::edit::{ChangeSet, MarkChanged};
use crate::*;

use super::super::lock::ensure_relation_unlocked_except;

pub(super) fn apply(
    project: &mut Project,
    sequence_id: &SequenceId,
    item_id: &ItemId,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let affected: Vec<_> = project
        .relations
        .iter()
        .filter(|relation| {
            relation.sequence_id == *sequence_id && incident(&relation.kind, item_id)
        })
        .cloned()
        .collect();
    let skipped = BTreeSet::from([item_id.clone()]);
    for relation in &affected {
        ensure_relation_unlocked_except(project, relation, &skipped, &BTreeSet::new())?;
    }
    let ids: BTreeSet<_> = affected
        .iter()
        .map(|relation| relation.id.clone())
        .collect();
    project
        .relations
        .retain(|relation| !ids.contains(&relation.id));
    for id in ids {
        changed.relation(id);
    }
    Ok(())
}

fn incident(kind: &RelationKind, item_id: &ItemId) -> bool {
    match kind {
        RelationKind::Transition { .. } => false,
        RelationKind::Matte {
            producer, consumer, ..
        } => producer.item_id() == Some(item_id) || consumer.item_id() == Some(item_id),
        RelationKind::Sidechain { target, .. } => target.item_id() == Some(item_id),
        RelationKind::Group { .. } | RelationKind::AvLink { .. } => false,
    }
}
