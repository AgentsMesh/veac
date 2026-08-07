use super::FrozenEntity;
use crate::program::expression::{ClosureValue, DomainOrigin, TemporalAttachmentKind, Value};

#[derive(Clone, Copy)]
pub(in crate::program) struct FrozenTemporalAttachment<'a> {
    graph: &'a super::super::FrozenDomainGraph,
    value: &'a super::super::state::TemporalAttachment,
}

impl<'a> FrozenTemporalAttachment<'a> {
    pub(super) fn new(
        graph: &'a super::super::FrozenDomainGraph,
        value: &'a super::super::state::TemporalAttachment,
    ) -> Self {
        Self { graph, value }
    }

    pub(in crate::program) fn owner(self) -> FrozenEntity<'a> {
        FrozenEntity::new(self.graph, self.value.owner)
    }

    pub(in crate::program) const fn kind(self) -> TemporalAttachmentKind {
        self.value.kind
    }

    pub(in crate::program) fn selectors(self) -> &'a [Value] {
        &self.value.selectors
    }

    pub(in crate::program) fn animation(self) -> &'a ClosureValue {
        &self.value.animation
    }

    pub(in crate::program) fn instance(self) -> Option<&'a DomainOrigin> {
        self.value.instance.as_ref()
    }
}

impl super::super::FrozenDomainGraph {
    pub(in crate::program) fn temporal_attachments(
        &self,
    ) -> impl Iterator<Item = FrozenTemporalAttachment<'_>> {
        self.state
            .temporal
            .iter()
            .map(|value| FrozenTemporalAttachment::new(self, value))
    }
}
