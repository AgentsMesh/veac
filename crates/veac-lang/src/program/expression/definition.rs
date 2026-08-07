use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::sync::Arc;

use super::core::{FunctionId, FunctionRegistry};
use super::{CompiledFunction, ValueType};

pub type TypeEnvironment = BTreeMap<String, ValueType>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionOrigin {
    source_id: String,
    body_span: Range<usize>,
}

impl FunctionOrigin {
    pub fn new(source_id: impl Into<String>, body_span: Range<usize>) -> Self {
        Self {
            source_id: source_id.into(),
            body_span,
        }
    }

    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub fn body_span(&self) -> Range<usize> {
        self.body_span.clone()
    }

    pub fn absolute_span(&self, relative: Range<usize>) -> Range<usize> {
        let body_end = self.body_span.end.max(self.body_span.start);
        let start = self
            .body_span
            .start
            .saturating_add(relative.start)
            .min(body_end);
        let end = self
            .body_span
            .start
            .saturating_add(relative.end)
            .min(body_end)
            .max(start);
        start..end
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionParameter {
    pub name: String,
    pub value_type: ValueType,
}

impl FunctionParameter {
    pub fn new(name: impl Into<String>, value_type: ValueType) -> Self {
        Self {
            name: name.into(),
            value_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDefinition {
    pub name: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: ValueType,
    pub body: String,
    pub origin: Option<FunctionOrigin>,
}

impl FunctionDefinition {
    pub fn new(
        name: impl Into<String>,
        parameters: Vec<FunctionParameter>,
        return_type: ValueType,
        body: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            parameters,
            return_type,
            body: body.into(),
            origin: None,
        }
    }

    pub fn with_origin(mut self, origin: FunctionOrigin) -> Self {
        self.origin = Some(origin);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct FunctionMap {
    visible: BTreeMap<String, FunctionId>,
    namespaces: BTreeSet<String>,
    registry: Arc<FunctionRegistry>,
}

impl FunctionMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lookup(&self, name: &str) -> Option<&Arc<CompiledFunction>> {
        self.lookup_id(name).and_then(|id| self.registry.get(id))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Arc<CompiledFunction>)> {
        self.visible
            .iter()
            .filter_map(|(name, id)| self.registry.get(*id).map(|value| (name.as_str(), value)))
    }

    pub fn len(&self) -> usize {
        self.visible.len()
    }

    pub fn is_empty(&self) -> bool {
        self.visible.is_empty()
    }

    pub(crate) fn register_namespace(&mut self, name: impl Into<String>) {
        self.namespaces.insert(name.into());
    }

    pub(crate) fn is_namespace(&self, name: &str) -> bool {
        self.namespaces.contains(name)
    }

    pub(crate) fn bind_from(
        &mut self,
        visible_name: impl Into<String>,
        source: &Self,
        source_name: &str,
    ) -> Option<FunctionId> {
        let id = source.lookup_id(source_name)?;
        Arc::make_mut(&mut self.registry).merge(source.registry.as_ref());
        self.visible.insert(visible_name.into(), id)
    }

    pub(super) fn insert(&mut self, function: Arc<CompiledFunction>) {
        let id = function.id();
        let name = function.name().to_owned();
        self.insert_hidden(function);
        self.visible.insert(name, id);
    }

    pub(crate) fn insert_hidden(&mut self, function: Arc<CompiledFunction>) {
        Arc::make_mut(&mut self.registry).insert(function);
    }

    pub(crate) fn merge_registry_from(&mut self, source: &Self) {
        Arc::make_mut(&mut self.registry).merge(source.registry.as_ref());
    }

    pub(crate) fn lookup_id(&self, name: &str) -> Option<FunctionId> {
        self.visible.get(name).copied()
    }

    pub(crate) fn registry(&self) -> &FunctionRegistry {
        self.registry.as_ref()
    }

    pub(crate) fn registry_arc(&self) -> Arc<FunctionRegistry> {
        Arc::clone(&self.registry)
    }

    pub(crate) fn lookup_by_id(&self, id: FunctionId) -> Option<&Arc<CompiledFunction>> {
        self.registry.get(id)
    }

    pub(crate) fn retain_visible_with_roots(
        &mut self,
        names: &BTreeSet<String>,
        additional_roots: impl IntoIterator<Item = FunctionId>,
    ) {
        self.visible.retain(|name, _| names.contains(name));
        let roots = self
            .visible
            .values()
            .copied()
            .chain(additional_roots)
            .collect::<Vec<_>>();
        self.registry = Arc::new(self.registry.reachable(&roots));
    }
}
