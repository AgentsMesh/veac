use std::mem::size_of;

use super::{BindingDependency, DependencyMask};
use crate::program::expression::core::metadata::ProjectionStep;
use crate::program::expression::InputId;

impl DependencyMask {
    pub(in crate::program::expression::core) fn retained_payload_bytes(&self) -> Option<usize> {
        let input_count = self.input_shape.len().checked_add(self.input_leaf.len())?;
        let binding_count = self
            .binding_shape
            .len()
            .checked_add(self.binding_leaf.len())?;
        let path_steps = checked_sum(
            self.binding_shape
                .iter()
                .chain(&self.binding_leaf)
                .map(|dependency| dependency.path.len()),
        )?;
        dependency_payload_bytes(input_count, binding_count, path_steps)
    }
}

fn dependency_payload_bytes(
    input_count: usize,
    binding_count: usize,
    path_steps: usize,
) -> Option<usize> {
    input_count
        .checked_mul(size_of::<InputId>())?
        .checked_add(binding_count.checked_mul(size_of::<BindingDependency>())?)?
        .checked_add(path_steps.checked_mul(size_of::<ProjectionStep>())?)
}

fn checked_sum(values: impl IntoIterator<Item = usize>) -> Option<usize> {
    values
        .into_iter()
        .try_fold(0usize, |total, value| total.checked_add(value))
}

#[cfg(test)]
#[path = "retained/tests.rs"]
mod tests;
