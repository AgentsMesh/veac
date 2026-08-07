use std::collections::BTreeMap;
use std::sync::Arc;

use super::super::{retained, TypeDefinition, TypeId, TypeRef, TypeRegistryError};
use super::{validation, TypeRegistry};
use crate::program::type_system::{MAX_TYPE_REGISTRY_BYTES, MAX_TYPE_REGISTRY_DEFINITIONS};

pub struct Builder {
    definitions: BTreeMap<TypeId, Arc<TypeDefinition>>,
    names: BTreeMap<String, TypeRef>,
    retained_bytes: usize,
    retained_limit: usize,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    pub fn new() -> Self {
        Self::with_limit(MAX_TYPE_REGISTRY_BYTES)
    }

    pub(crate) fn with_limit(retained_limit: usize) -> Self {
        Self {
            definitions: BTreeMap::new(),
            names: BTreeMap::new(),
            retained_bytes: 0,
            retained_limit,
        }
    }

    pub fn merge_definitions(&mut self, registry: &TypeRegistry) -> Result<(), TypeRegistryError> {
        for definition in registry.definitions.values() {
            self.insert(Arc::clone(definition))?;
        }
        Ok(())
    }

    pub fn merge(&mut self, registry: &TypeRegistry) -> Result<(), TypeRegistryError> {
        self.merge_definitions(registry)?;
        for (name, value) in registry.names() {
            self.bind(name, value.clone())?;
        }
        Ok(())
    }

    pub fn resolve(&self, name: &str) -> Option<&TypeRef> {
        self.names.get(name)
    }

    pub fn insert(&mut self, definition: Arc<TypeDefinition>) -> Result<(), TypeRegistryError> {
        validation::definition(&definition)?;
        let id = definition.type_ref().id();
        if let Some(existing) = self.definitions.get(&id) {
            return (existing.as_ref() == definition.as_ref())
                .then_some(())
                .ok_or_else(|| collision(id));
        }
        if self.definitions.len() >= MAX_TYPE_REGISTRY_DEFINITIONS {
            return Err(limit("type registry definition count exceeds 1024"));
        }
        let added = retained::definition(&definition)
            .ok_or_else(|| limit("type registry retained storage overflows"))?;
        self.charge(added)?;
        self.definitions.insert(id, definition);
        Ok(())
    }

    pub fn bind(
        &mut self,
        name: impl Into<String>,
        value: TypeRef,
    ) -> Result<(), TypeRegistryError> {
        let name = name.into();
        if !crate::name::is_qualified_name(&name) {
            return Err(TypeRegistryError::new(
                "TYPE_INVALID_NAME",
                format!("type alias `{name}` is not a valid qualified name"),
            ));
        }
        if crate::program::DomainType::parse(&name).is_some() {
            return Err(TypeRegistryError::new(
                "TYPE_RESERVED_NAME",
                format!("type name `{name}` is reserved by the domain language"),
            ));
        }
        if self.names.contains_key(&name) {
            return Err(TypeRegistryError::new(
                "TYPE_DUPLICATE_NAME",
                format!("type name `{name}` is bound more than once"),
            ));
        }
        let added = retained::binding(&name, &value)
            .ok_or_else(|| limit("type registry retained storage overflows"))?;
        self.charge(added)?;
        self.names.insert(name, value);
        Ok(())
    }

    pub fn finish(self) -> Result<TypeRegistry, TypeRegistryError> {
        for (name, value) in &self.names {
            if !self.definitions.contains_key(&value.id()) {
                return Err(TypeRegistryError::new(
                    "TYPE_UNKNOWN_ID",
                    format!("type name `{name}` refers to unknown TypeId {}", value.id()),
                ));
            }
        }
        validation::registry(&self.definitions)?;
        Ok(TypeRegistry {
            definitions: self.definitions,
            names: self.names,
            retained_bytes: self.retained_bytes,
        })
    }

    fn charge(&mut self, added: usize) -> Result<(), TypeRegistryError> {
        self.retained_bytes = self
            .retained_bytes
            .checked_add(added)
            .filter(|value| *value <= self.retained_limit)
            .ok_or_else(|| limit("type registry exceeds its retained storage limit"))?;
        Ok(())
    }
}

fn collision(id: TypeId) -> TypeRegistryError {
    TypeRegistryError::new(
        "TYPE_ID_COLLISION",
        format!("TypeId {id} has conflicting definitions"),
    )
}

fn limit(message: &'static str) -> TypeRegistryError {
    TypeRegistryError::new("TYPE_REGISTRY_LIMIT", message)
}
