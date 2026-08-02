use crate::edit::{ChangeSet, MarkChanged};
use crate::*;

use super::ItemTopology;

pub(super) fn rewrite(
    relations: &mut Vec<Relation>,
    sequence_id: &SequenceId,
    topology: &[ItemTopology],
    changed: &mut ChangeSet,
) {
    relations.retain_mut(|relation| {
        if relation.sequence_id != *sequence_id {
            return true;
        }
        let RelationKind::Transition { from, to, .. } = &mut relation.kind else {
            return true;
        };
        let outgoing = from.item_id();
        let incoming = to.item_id();
        let remove = topology.iter().any(|change| match change {
            ItemTopology::Removed { item_id } => {
                outgoing == Some(item_id) || incoming == Some(item_id)
            }
            ItemTopology::KeptLeft { item_id } => outgoing == Some(item_id),
            ItemTopology::KeptRight { item_id, .. } => incoming == Some(item_id),
            ItemTopology::Split { .. } => false,
        });
        if remove {
            changed.relation(relation.id.clone());
            return false;
        }
        if let Some(right_id) = topology.iter().find_map(|change| match change {
            ItemTopology::Split {
                item_id,
                right_item_id,
                ..
            } if outgoing == Some(item_id) => Some(right_item_id),
            _ => None,
        }) {
            *from = RelationEndpoint::item(right_id.clone());
            changed.relation(relation.id.clone());
        }
        true
    });
}
