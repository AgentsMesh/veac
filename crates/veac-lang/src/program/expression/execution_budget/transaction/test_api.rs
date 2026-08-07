use super::super::resource::Resource;
use super::super::{ExecutionBudget, ResourceDelta};

impl ExecutionBudget {
    pub(crate) fn usage(&self) -> ResourceDelta {
        ResourceDelta {
            fuel: self.used(Resource::Fuel),
            value_bytes: self.used(Resource::ValueBytes),
            iterations: self.used(Resource::Iterations),
            collection_elements: self.used(Resource::CollectionElements),
            collection_bytes: self.used(Resource::CollectionBytes),
            emitted_entities: self.used(Resource::EmittedEntities),
            emitted_bytes: self.used(Resource::EmittedBytes),
            residual_nodes: self.used(Resource::ResidualNodes),
            residual_bytes: self.used(Resource::ResidualBytes),
            evaluator_storage_bytes: self.used(Resource::EvaluatorStorageBytes),
        }
    }

    fn used(&self, resource: Resource) -> usize {
        self.counters[resource.index()].used.get()
    }
}
