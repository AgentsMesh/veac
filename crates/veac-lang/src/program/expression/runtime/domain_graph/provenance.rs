use crate::program::expression::{self, DomainOrigin};
use crate::program::DomainOperationId;
use veac_ir::{AuthorshipEvent, AuthorshipEventKind, EntityAuthorship, LogicalPathSegment};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProvenanceKind {
    Construct,
    Constructor,
    Update,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DomainProvenance {
    origin: DomainOrigin,
    kind: ProvenanceKind,
}

impl DomainProvenance {
    pub(super) fn new(origin: DomainOrigin, kind: ProvenanceKind) -> Self {
        Self { origin, kind }
    }

    pub(super) fn event(&self, operation: DomainOperationId) -> AuthorshipEvent {
        self.origin.event(operation, self.kind.authorship())
    }

    pub(super) fn logical_bytes(
        &self,
        operation: DomainOperationId,
        entity_key: Option<&str>,
    ) -> usize {
        let event = expression::provenance::json_bytes(&self.event(operation));
        match self.kind {
            ProvenanceKind::Constructor => entity_key.map_or(event, |key| {
                event
                    .saturating_add(2)
                    .saturating_add(path_bytes(&[key]))
                    .saturating_add(entity_envelope_overhead())
            }),
            ProvenanceKind::Update => event.saturating_add(1),
            ProvenanceKind::Construct => event,
        }
    }
}

pub(super) fn path_extension_bytes(key: &str, entities: usize) -> Option<usize> {
    let key = expression::provenance::json_bytes(&LogicalPathSegment::new(key));
    key.checked_add(1)?.checked_mul(entities)
}

fn path_bytes(path: &[&str]) -> usize {
    expression::provenance::json_bytes(
        &path
            .iter()
            .map(|value| LogicalPathSegment::new(*value))
            .collect::<Vec<_>>(),
    )
}

fn entity_envelope_overhead() -> usize {
    let value = EntityAuthorship {
        events: Vec::new(),
        logical_path: Vec::new(),
    };
    expression::provenance::json_bytes(&value) - 4
}

impl ProvenanceKind {
    const fn authorship(self) -> AuthorshipEventKind {
        match self {
            Self::Construct => AuthorshipEventKind::Construct,
            Self::Constructor => AuthorshipEventKind::Constructor,
            Self::Update => AuthorshipEventKind::Update,
        }
    }
}
