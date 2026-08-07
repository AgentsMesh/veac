use veac_ir::{EntityAuthorship, LogicalPathSegment};

use super::FrozenEntity;

impl FrozenEntity<'_> {
    pub(in crate::program) fn provenance(self) -> Option<EntityAuthorship> {
        let node = self.graph.state.nodes.get(self.node.0)?;
        let events = self
            .graph
            .state
            .records
            .iter()
            .skip(node.origin_slot as usize)
            .take((node.latest_slot - node.origin_slot + 1) as usize)
            .filter(|record| record.node == self.node)
            .filter_map(|record| Some(record.provenance.as_ref()?.event(record.operation?)))
            .collect::<Vec<_>>();
        if events.is_empty() {
            return None;
        }
        Some(EntityAuthorship {
            logical_path: typed_path(self)?,
            events,
        })
    }
}

fn typed_path(value: FrozenEntity<'_>) -> Option<Vec<LogicalPathSegment>> {
    let mut nodes = Vec::new();
    let mut cursor = Some(value.node);
    for _ in 0..=value.graph.state.nodes.len() {
        let node = &value.graph.state.nodes[cursor?.0];
        nodes.push((node.domain_type, node.key.as_deref()?));
        cursor = node.owner;
        if cursor.is_none() {
            nodes.reverse();
            let mut path = Vec::with_capacity(nodes.len().saturating_mul(2).saturating_sub(1));
            for (index, (domain_type, key)) in nodes.into_iter().enumerate() {
                if index > 0 {
                    path.push(LogicalPathSegment::new(
                        domain_type.name().to_ascii_lowercase(),
                    ));
                }
                path.push(LogicalPathSegment::new(key));
            }
            return Some(path);
        }
    }
    None
}
