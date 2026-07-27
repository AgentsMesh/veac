use crate::edit::{ChangeSet, MarkChanged};
use crate::*;

use super::ensure_relation_unlocked;

pub(in crate::edit::operations) fn remove(
    project: &mut Project,
    sequence_id: &SequenceId,
    apply_id: &ApplyId,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let affected: Vec<_> = project
        .relations
        .iter()
        .filter(|relation| relation.sequence_id == *sequence_id)
        .filter(|relation| {
            matches!(
                &relation.kind,
                RelationKind::Matte {
                    consumer: RelationEndpoint::Apply { apply_id: id },
                    ..
                } if id == apply_id
            )
        })
        .cloned()
        .collect();
    for relation in &affected {
        ensure_relation_unlocked(project, relation)?;
    }
    let ids: std::collections::BTreeSet<_> =
        affected.into_iter().map(|relation| relation.id).collect();
    project
        .relations
        .retain(|relation| !ids.contains(&relation.id));
    if !ids.is_empty() {
        changed.project(project.id.clone());
        for id in ids {
            changed.relation(id);
        }
    }
    Ok(())
}
