use std::mem::{size_of, size_of_val};

use super::super::FunctionParameter;
use super::body::ResolvedBody;
use super::query::{CompiledFunctionBatch, TypedFunctionBatch};

impl TypedFunctionBatch {
    pub(crate) fn retained_bytes(&self) -> Option<usize> {
        let shared = self
            .bodies()
            .first()
            .map_or(0, |body| body.typed().cache_shared_bytes());
        let initial = size_of::<Self>()
            .checked_add(shared)?
            .checked_add(self.order().len().checked_mul(size_of::<usize>())?);
        self.bodies().iter().try_fold(initial?, |bytes, body| {
            bytes
                .checked_add(body_bytes(body)?)
                .and_then(|value| value.checked_add(body.typed().cache_tree_bytes()))
        })
    }
}

impl CompiledFunctionBatch {
    pub(crate) fn cache_retained_bytes(&self) -> usize {
        let visible = self
            .functions()
            .visible_bindings()
            .fold(0usize, |bytes, (name, _)| {
                bytes
                    .saturating_add(size_of::<String>())
                    .saturating_add(name.len())
                    .saturating_add(size_of::<super::super::FunctionId>())
                    .saturating_add(128)
            });
        let namespaces = self.functions().namespaces().fold(0usize, |bytes, name| {
            bytes
                .saturating_add(size_of::<String>())
                .saturating_add(name.len())
                .saturating_add(128)
        });
        let registry =
            self.functions()
                .registry_functions()
                .fold(0usize, |bytes, (_, function)| {
                    bytes
                        .saturating_add(size_of_val(function))
                        .saturating_add(function.retained_bytes().unwrap_or(usize::MAX))
                        .saturating_add(128)
                });
        size_of::<Self>()
            .saturating_add(visible)
            .saturating_add(namespaces)
            .saturating_add(registry)
    }
}

fn body_bytes(value: &ResolvedBody) -> Option<usize> {
    let mut bytes = size_of_val(value)
        .checked_add(value.source().len())?
        .checked_add(value.name().len())?
        .checked_add(super::super::value_type::retained_shape_bytes(
            value.return_type(),
        )?)?;
    if let Some(origin) = value.origin() {
        let span = origin.body_span();
        bytes = bytes
            .checked_add(origin.source_id().len())?
            .checked_add(size_of_val(&span))?;
    }
    for parameter in value.parameters() {
        bytes = bytes.checked_add(parameter_bytes(parameter)?)?;
    }
    Some(bytes)
}

fn parameter_bytes(value: &FunctionParameter) -> Option<usize> {
    let mut bytes = size_of_val(value)
        .checked_add(value.name.len())?
        .checked_add(super::super::value_type::retained_shape_bytes(
            &value.value_type,
        )?)?;
    if let Some(default) = value.default() {
        bytes = bytes.checked_add(default.source().len())?;
        if let Some(origin) = default.origin() {
            let span = origin.body_span();
            bytes = bytes
                .checked_add(origin.source_id().len())?
                .checked_add(size_of_val(&span))?;
        }
    }
    Some(bytes)
}
