use crate::edit::{ChangeSet, MarkChanged};
use crate::*;

mod apply;
mod lock;
mod signal;
mod track;
mod transition;

pub(super) use apply::remove as remove_apply;
pub(super) use lock::ensure_relation_unlocked_except;

pub(super) use lock::ensure_relation_unlocked;
pub(super) use track::{prune_disappeared_bus, remove_sequence, remove_track};

#[derive(Clone)]
pub(super) enum ItemTopology {
    Removed {
        item_id: ItemId,
    },
    KeptLeft {
        item_id: ItemId,
    },
    KeptRight {
        item_id: ItemId,
        offset: RationalTime,
    },
    Split {
        item_id: ItemId,
        right_item_id: ItemId,
        relation_fragments: Vec<RelationFragmentId>,
    },
}

pub(super) fn rewrite_item_relations(
    project: &mut Project,
    sequence_id: &SequenceId,
    topology: &[ItemTopology],
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let before = project.relations.clone();
    transition::rewrite(&mut project.relations, sequence_id, topology, changed);
    signal::rewrite(project, sequence_id, topology, changed)?;
    if project.relations != before {
        changed.project(project.id.clone());
    }
    Ok(())
}
