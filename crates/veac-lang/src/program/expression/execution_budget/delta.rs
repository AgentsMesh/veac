#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResourceDelta {
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
