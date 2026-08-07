#[path = "tests/aggregate.rs"]
mod aggregate;
#[path = "tests/aggregate_runtime.rs"]
mod aggregate_runtime;
#[path = "tests/atomic.rs"]
mod atomic;
#[path = "tests/boundaries.rs"]
mod boundaries;
#[path = "tests/closure.rs"]
mod closure;
#[path = "tests/closure_overflow.rs"]
mod closure_overflow;
#[path = "tests/collection.rs"]
mod collection;
#[path = "tests/domain_graph.rs"]
mod domain_graph;
#[path = "tests/domain_iteration.rs"]
mod domain_iteration;
#[path = "tests/domain_metadata.rs"]
mod domain_metadata;
#[path = "tests/evaluator.rs"]
mod evaluator;
#[path = "tests/nominal.rs"]
mod nominal;
#[path = "tests/range.rs"]
mod range;
#[path = "tests/structural.rs"]
mod structural;

use super::limits::ResourceLimits;
use super::resource::Resource;
use super::{ExecutionBudget, ResourceDelta};

fn budget(resource: Resource, limit: usize) -> ExecutionBudget {
    let mut limits = unlimited();
    set_limit(&mut limits, resource, limit);
    ExecutionBudget::with_resource_limits(limits)
}

fn unlimited() -> ResourceLimits {
    ResourceLimits {
        fuel: usize::MAX,
        value_bytes: usize::MAX,
        iterations: usize::MAX,
        collection_elements: usize::MAX,
        collection_bytes: usize::MAX,
        emitted_entities: usize::MAX,
        emitted_bytes: usize::MAX,
        residual_nodes: usize::MAX,
        residual_bytes: usize::MAX,
        evaluator_storage_bytes: usize::MAX,
    }
}

fn set_limit(limits: &mut ResourceLimits, resource: Resource, limit: usize) {
    match resource {
        Resource::Fuel => limits.fuel = limit,
        Resource::ValueBytes => limits.value_bytes = limit,
        Resource::Iterations => limits.iterations = limit,
        Resource::CollectionElements => limits.collection_elements = limit,
        Resource::CollectionBytes => limits.collection_bytes = limit,
        Resource::EmittedEntities => limits.emitted_entities = limit,
        Resource::EmittedBytes => limits.emitted_bytes = limit,
        Resource::ResidualNodes => limits.residual_nodes = limit,
        Resource::ResidualBytes => limits.residual_bytes = limit,
        Resource::EvaluatorStorageBytes => limits.evaluator_storage_bytes = limit,
    }
}

fn delta(resource: Resource, amount: usize) -> ResourceDelta {
    let mut delta = ResourceDelta::default();
    match resource {
        Resource::Fuel => delta.fuel = amount,
        Resource::ValueBytes => delta.value_bytes = amount,
        Resource::Iterations => delta.iterations = amount,
        Resource::CollectionElements => delta.collection_elements = amount,
        Resource::CollectionBytes => delta.collection_bytes = amount,
        Resource::EmittedEntities => delta.emitted_entities = amount,
        Resource::EmittedBytes => delta.emitted_bytes = amount,
        Resource::ResidualNodes => delta.residual_nodes = amount,
        Resource::ResidualBytes => delta.residual_bytes = amount,
        Resource::EvaluatorStorageBytes => delta.evaluator_storage_bytes = amount,
    }
    delta
}

fn used(budget: &ExecutionBudget, resource: Resource) -> usize {
    budget.counters[resource.index()].used.get()
}
