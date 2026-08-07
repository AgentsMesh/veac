use super::{BindingRoot, MetadataPath};
use crate::program::expression::InputId;

mod retained;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct BindingDependency {
    pub(super) root: BindingRoot,
    pub(super) path: MetadataPath,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DependencyMask {
    binding_shape: Vec<BindingDependency>,
    binding_leaf: Vec<BindingDependency>,
    input_shape: Vec<InputId>,
    input_leaf: Vec<InputId>,
}

impl DependencyMask {
    pub fn is_empty(&self) -> bool {
        self.binding_shape.is_empty()
            && self.binding_leaf.is_empty()
            && self.input_shape.is_empty()
            && self.input_leaf.is_empty()
    }

    pub fn depends_on_parameter_shape(&self, index: usize) -> bool {
        contains_binding(&self.binding_shape, BindingRoot::Parameter(index))
    }

    pub fn depends_on_parameter_leaf(&self, index: usize) -> bool {
        contains_binding(&self.binding_leaf, BindingRoot::Parameter(index))
    }

    pub fn depends_on_capture_shape(&self, index: usize) -> bool {
        contains_binding(&self.binding_shape, BindingRoot::Capture(index))
    }

    pub fn depends_on_capture_leaf(&self, index: usize) -> bool {
        contains_binding(&self.binding_leaf, BindingRoot::Capture(index))
    }

    pub fn shape_input_ids(&self) -> &[InputId] {
        &self.input_shape
    }

    pub fn leaf_input_ids(&self) -> &[InputId] {
        &self.input_leaf
    }

    pub(crate) fn parameter_shape(index: usize) -> Self {
        Self::binding_shape(BindingRoot::Parameter(index), MetadataPath::default())
    }

    pub(crate) fn parameter_leaf(index: usize) -> Self {
        Self::binding_leaf(BindingRoot::Parameter(index), MetadataPath::default())
    }

    pub(crate) fn capture_shape(index: usize) -> Self {
        Self::binding_shape(BindingRoot::Capture(index), MetadataPath::default())
    }

    pub(crate) fn capture_leaf(index: usize) -> Self {
        Self::binding_leaf(BindingRoot::Capture(index), MetadataPath::default())
    }

    pub(crate) fn binding_shape(root: BindingRoot, path: MetadataPath) -> Self {
        Self {
            binding_shape: vec![BindingDependency { root, path }],
            ..Self::default()
        }
    }

    pub(crate) fn binding_leaf(root: BindingRoot, path: MetadataPath) -> Self {
        Self {
            binding_leaf: vec![BindingDependency { root, path }],
            ..Self::default()
        }
    }

    pub(crate) fn input_shape(id: InputId) -> Self {
        Self {
            input_shape: vec![id],
            ..Self::default()
        }
    }

    pub(crate) fn input_leaf(id: InputId) -> Self {
        Self {
            input_leaf: vec![id],
            ..Self::default()
        }
    }

    pub(crate) fn union(&mut self, other: &Self) {
        union_sorted(&mut self.binding_shape, &other.binding_shape);
        union_sorted(&mut self.binding_leaf, &other.binding_leaf);
        union_inputs(&mut self.input_shape, &other.input_shape);
        union_inputs(&mut self.input_leaf, &other.input_leaf);
    }

    pub(crate) fn contains(&self, other: &Self) -> bool {
        contains_sorted(&self.binding_shape, &other.binding_shape)
            && contains_sorted(&self.binding_leaf, &other.binding_leaf)
            && contains_inputs(&self.input_shape, &other.input_shape)
            && contains_inputs(&self.input_leaf, &other.input_leaf)
    }

    pub(super) fn without_bindings(&self) -> Self {
        Self {
            input_shape: self.input_shape.clone(),
            input_leaf: self.input_leaf.clone(),
            ..Self::default()
        }
    }

    pub(super) fn shape_bindings(&self) -> &[BindingDependency] {
        &self.binding_shape
    }

    pub(super) fn leaf_bindings(&self) -> &[BindingDependency] {
        &self.binding_leaf
    }
}

fn contains_binding(values: &[BindingDependency], root: BindingRoot) -> bool {
    values.iter().any(|value| value.root == root)
}

fn union_inputs(output: &mut Vec<InputId>, inputs: &[InputId]) {
    for input in inputs {
        if let Err(index) = output.binary_search(input) {
            output.insert(index, *input);
        }
    }
}

fn contains_inputs(values: &[InputId], required: &[InputId]) -> bool {
    required
        .iter()
        .all(|input| values.binary_search(input).is_ok())
}

fn union_sorted<T: Clone + Ord>(output: &mut Vec<T>, values: &[T]) {
    for value in values {
        if let Err(index) = output.binary_search(value) {
            output.insert(index, value.clone());
        }
    }
}

fn contains_sorted<T: Ord>(values: &[T], required: &[T]) -> bool {
    required
        .iter()
        .all(|value| values.binary_search(value).is_ok())
}
