use super::delta::ResourceDelta;
use super::limits::ResourceLimits;

pub(super) const RESOURCE_COUNT: usize = 10;

pub(super) const RESOURCES: [Resource; RESOURCE_COUNT] = [
    Resource::Fuel,
    Resource::ValueBytes,
    Resource::Iterations,
    Resource::CollectionElements,
    Resource::CollectionBytes,
    Resource::EmittedEntities,
    Resource::EmittedBytes,
    Resource::ResidualNodes,
    Resource::ResidualBytes,
    Resource::EvaluatorStorageBytes,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub(super) enum Resource {
    Fuel,
    ValueBytes,
    Iterations,
    CollectionElements,
    CollectionBytes,
    EmittedEntities,
    EmittedBytes,
    ResidualNodes,
    ResidualBytes,
    EvaluatorStorageBytes,
}

impl Resource {
    pub(super) const fn index(self) -> usize {
        self as usize
    }

    pub(super) const fn amount(self, delta: &ResourceDelta) -> usize {
        match self {
            Self::Fuel => delta.fuel,
            Self::ValueBytes => delta.value_bytes,
            Self::Iterations => delta.iterations,
            Self::CollectionElements => delta.collection_elements,
            Self::CollectionBytes => delta.collection_bytes,
            Self::EmittedEntities => delta.emitted_entities,
            Self::EmittedBytes => delta.emitted_bytes,
            Self::ResidualNodes => delta.residual_nodes,
            Self::ResidualBytes => delta.residual_bytes,
            Self::EvaluatorStorageBytes => delta.evaluator_storage_bytes,
        }
    }

    pub(super) const fn limit(self, limits: &ResourceLimits) -> usize {
        match self {
            Self::Fuel => limits.fuel,
            Self::ValueBytes => limits.value_bytes,
            Self::Iterations => limits.iterations,
            Self::CollectionElements => limits.collection_elements,
            Self::CollectionBytes => limits.collection_bytes,
            Self::EmittedEntities => limits.emitted_entities,
            Self::EmittedBytes => limits.emitted_bytes,
            Self::ResidualNodes => limits.residual_nodes,
            Self::ResidualBytes => limits.residual_bytes,
            Self::EvaluatorStorageBytes => limits.evaluator_storage_bytes,
        }
    }

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Fuel => "fuel",
            Self::ValueBytes => "evaluated value byte",
            Self::Iterations => "aggregate iteration",
            Self::CollectionElements => "aggregate collection element",
            Self::CollectionBytes => "collection byte",
            Self::EmittedEntities => "emitted entity",
            Self::EmittedBytes => "emitted byte",
            Self::ResidualNodes => "residual program node",
            Self::ResidualBytes => "residual program byte",
            Self::EvaluatorStorageBytes => "evaluator storage byte",
        }
    }
}
