use super::{identity, DomainType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DomainTypeClassification {
    Container,
    GraphEntity,
    TopologyValue,
    LeafValue,
}

impl DomainTypeClassification {
    const fn is_container(self) -> bool {
        matches!(self, Self::Container)
    }

    const fn is_graph_entity(self) -> bool {
        matches!(self, Self::Container | Self::GraphEntity)
    }

    const fn requires_topology_axis(self) -> bool {
        !matches!(self, Self::LeafValue)
    }
}

impl DomainType {
    pub const fn is_container(self) -> bool {
        classification(self).is_container()
    }

    pub const fn is_graph_entity(self) -> bool {
        classification(self).is_graph_entity()
    }

    pub const fn requires_topology_axis(self) -> bool {
        classification(self).requires_topology_axis()
    }
}

const fn classification(value: DomainType) -> DomainTypeClassification {
    match identity::find(value.opcode()) {
        Some(identity) => identity.classification(),
        None => panic!("invalid closed domain type"),
    }
}
