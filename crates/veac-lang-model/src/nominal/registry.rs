use std::collections::BTreeMap;
use std::sync::Arc;

use super::{TypeDefinition, TypeId, TypeRef};

mod builder;
mod layout;
mod validation;

pub use builder::Builder;

#[derive(Debug, Clone, Default)]
pub struct TypeRegistry {
    definitions: BTreeMap<TypeId, Arc<TypeDefinition>>,
    names: BTreeMap<String, TypeRef>,
    retained_bytes: usize,
}

impl TypeRegistry {
    pub fn resolve(&self, name: &str) -> Option<&TypeRef> {
        self.names.get(name)
    }

    pub fn definition(&self, id: TypeId) -> Option<&TypeDefinition> {
        self.definitions.get(&id).map(Arc::as_ref)
    }

    #[doc(hidden)]
    pub fn definition_handle(&self, id: TypeId) -> Option<Arc<TypeDefinition>> {
        self.definitions.get(&id).map(Arc::clone)
    }

    pub fn definitions(&self) -> impl ExactSizeIterator<Item = &TypeDefinition> {
        self.definitions.values().map(Arc::as_ref)
    }

    pub fn names(&self) -> impl ExactSizeIterator<Item = (&str, &TypeRef)> {
        self.names
            .iter()
            .map(|(name, value)| (name.as_str(), value))
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    pub const fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    pub fn contains_function(&self, id: TypeId) -> Option<bool> {
        layout::contains_function(&self.definitions, id)
    }
}
