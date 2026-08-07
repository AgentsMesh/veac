use super::super::DomainOperationId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TemporalLoweringOpcode {
    ComposeVector,
    ComposePoint,
    ComposeRect,
}

impl TemporalLoweringOpcode {
    pub(super) const fn for_operation(id: DomainOperationId) -> Option<Self> {
        match id {
            DomainOperationId::Vector => Some(Self::ComposeVector),
            DomainOperationId::Point => Some(Self::ComposePoint),
            DomainOperationId::Rect => Some(Self::ComposeRect),
            _ => None,
        }
    }
}
