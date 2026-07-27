use crate::edit::ChangeSet;
use crate::*;

use super::ItemTopology;

mod fragment;
mod range;
mod remove;
mod split;
mod trim;

pub(super) fn rewrite(
    project: &mut Project,
    sequence_id: &SequenceId,
    topology: &[ItemTopology],
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let zero = RationalTime::new(0, project.timebase).unwrap();
    for change in topology {
        match change {
            ItemTopology::Removed { item_id } => {
                remove::apply(project, sequence_id, item_id, changed)?
            }
            ItemTopology::Split {
                item_id,
                right_item_id,
                relation_fragments,
            } => split::apply(
                project,
                sequence_id,
                item_id,
                right_item_id,
                relation_fragments,
                changed,
            )?,
            ItemTopology::KeptLeft { item_id } => {
                trim::apply(project, sequence_id, item_id, zero, changed)?
            }
            ItemTopology::KeptRight { item_id, offset } => {
                trim::apply(project, sequence_id, item_id, *offset, changed)?
            }
        }
    }
    Ok(())
}
