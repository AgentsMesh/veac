use super::{
    MAX_COLLECTION_BYTES, MAX_COLLECTION_ELEMENTS, MAX_EMITTED_BYTES, MAX_EMITTED_ENTITIES,
    MAX_EVALUATED_VALUE_BYTES, MAX_EVALUATOR_STORAGE_BYTES, MAX_EXECUTION_FUEL,
    MAX_EXECUTION_ITERATIONS, MAX_RESIDUAL_BYTES, MAX_RESIDUAL_NODES,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResourceLimits {
    pub(crate) fuel: usize,
    pub(crate) value_bytes: usize,
    pub(crate) iterations: usize,
    pub(crate) collection_elements: usize,
    pub(crate) collection_bytes: usize,
    pub(crate) emitted_entities: usize,
    pub(crate) emitted_bytes: usize,
    pub(crate) residual_nodes: usize,
    pub(crate) residual_bytes: usize,
    pub(crate) evaluator_storage_bytes: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            fuel: MAX_EXECUTION_FUEL,
            value_bytes: MAX_EVALUATED_VALUE_BYTES,
            iterations: MAX_EXECUTION_ITERATIONS,
            collection_elements: MAX_COLLECTION_ELEMENTS,
            collection_bytes: MAX_COLLECTION_BYTES,
            emitted_entities: MAX_EMITTED_ENTITIES,
            emitted_bytes: MAX_EMITTED_BYTES,
            residual_nodes: MAX_RESIDUAL_NODES,
            residual_bytes: MAX_RESIDUAL_BYTES,
            evaluator_storage_bytes: MAX_EVALUATOR_STORAGE_BYTES,
        }
    }
}
