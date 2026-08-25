use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::program::expression::FunctionId;
use crate::program::{TypeId, TypeRegistry};

use super::{
    retained, MethodDefinition, MethodRegistryError, MethodVisibility, MAX_METHODS_PER_TYPE,
    MAX_METHOD_REGISTRY_BYTES, MAX_METHOD_REGISTRY_DEFINITIONS,
};

mod validation;

type Definitions = BTreeMap<TypeId, BTreeMap<String, Arc<MethodDefinition>>>;

#[derive(Debug, Clone, Default)]
pub struct MethodRegistry {
    definitions: Definitions,
    retained_bytes: usize,
}

pub struct Builder {
    definitions: Definitions,
    ids: BTreeMap<FunctionId, (TypeId, String)>,
    retained_bytes: usize,
    retained_limit: usize,
}

impl MethodRegistry {
    pub fn lookup(&self, receiver: TypeId, name: &str) -> Option<&MethodDefinition> {
        self.definitions.get(&receiver)?.get(name).map(Arc::as_ref)
    }

    pub fn definitions(&self) -> impl Iterator<Item = &MethodDefinition> {
        self.definitions
            .values()
            .flat_map(|methods| methods.values().map(Arc::as_ref))
    }

    pub fn function_ids(&self) -> impl Iterator<Item = FunctionId> + '_ {
        self.definitions()
            .map(|definition| definition.signature().function_id())
    }

    pub fn len(&self) -> usize {
        self.definitions.values().map(BTreeMap::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    pub const fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    pub fn exported_for(&self, receivers: &BTreeSet<TypeId>) -> Self {
        self.filtered(
            |definition| {
                definition.visibility() == MethodVisibility::Exported
                    && receivers.contains(&definition.signature().receiver().id())
            },
            true,
        )
    }

    pub fn reachable(&self, roots: impl IntoIterator<Item = FunctionId>) -> Self {
        let roots = roots.into_iter().collect::<BTreeSet<_>>();
        self.filtered(
            |definition| roots.contains(&definition.signature().function_id()),
            false,
        )
    }

    fn filtered(&self, keep: impl Fn(&MethodDefinition) -> bool, interface: bool) -> Self {
        let mut definitions: Definitions = BTreeMap::new();
        let mut retained_bytes = 0usize;
        for definition in self.definitions().filter(|value| keep(value)) {
            let value = if interface {
                definition.exported_interface()
            } else {
                definition.clone()
            };
            retained_bytes += retained::definition(&value).expect("admitted size remains valid");
            definitions
                .entry(value.signature().receiver().id())
                .or_default()
                .insert(value.signature().name().to_owned(), Arc::new(value));
        }
        Self {
            definitions,
            retained_bytes,
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    pub fn new() -> Self {
        Self::with_limit(MAX_METHOD_REGISTRY_BYTES)
    }

    pub(crate) fn with_limit(retained_limit: usize) -> Self {
        Self {
            definitions: BTreeMap::new(),
            ids: BTreeMap::new(),
            retained_bytes: 0,
            retained_limit,
        }
    }

    pub fn merge(
        &mut self,
        registry: &MethodRegistry,
        types: &TypeRegistry,
    ) -> Result<(), MethodRegistryError> {
        for definition in registry.definitions.values().flat_map(BTreeMap::values) {
            self.insert(Arc::clone(definition), types)?;
        }
        Ok(())
    }

    pub fn insert(
        &mut self,
        definition: Arc<MethodDefinition>,
        types: &TypeRegistry,
    ) -> Result<(), MethodRegistryError> {
        validation::definition(&definition, types)?;
        let signature = definition.signature();
        let receiver = signature.receiver().id();
        let name = signature.name().to_owned();
        if let Some(existing) = self
            .definitions
            .get(&receiver)
            .and_then(|map| map.get(&name))
        {
            return (existing.as_ref() == definition.as_ref())
                .then_some(())
                .ok_or_else(|| duplicate(receiver, &name));
        }
        if self.definitions.get(&receiver).map_or(0, BTreeMap::len) >= MAX_METHODS_PER_TYPE {
            return Err(limit("nominal type exceeds the 64 method limit"));
        }
        if self.ids.len() >= MAX_METHOD_REGISTRY_DEFINITIONS {
            return Err(limit("method registry definition count exceeds 1024"));
        }
        let id = signature.function_id();
        if let Some((other_receiver, other_name)) = self.ids.get(&id) {
            return Err(MethodRegistryError::new(
                "METHOD_FUNCTION_ID_COLLISION",
                format!(
                    "FunctionId {id} is shared by methods {other_receiver}.{other_name} and {receiver}.{name}"
                ),
            ));
        }
        let added = retained::definition(&definition)
            .ok_or_else(|| limit("method registry retained storage overflows"))?;
        self.retained_bytes = self
            .retained_bytes
            .checked_add(added)
            .filter(|value| *value <= self.retained_limit)
            .ok_or_else(|| limit("method registry exceeds its retained storage limit"))?;
        self.ids.insert(id, (receiver, name.clone()));
        self.definitions
            .entry(receiver)
            .or_default()
            .insert(name, definition);
        Ok(())
    }

    pub fn finish(self) -> MethodRegistry {
        MethodRegistry {
            definitions: self.definitions,
            retained_bytes: self.retained_bytes,
        }
    }
}

fn duplicate(receiver: TypeId, name: &str) -> MethodRegistryError {
    MethodRegistryError::new(
        "METHOD_DUPLICATE",
        format!("method {receiver}.{name} is defined more than once"),
    )
}

fn limit(message: &'static str) -> MethodRegistryError {
    MethodRegistryError::new("METHOD_REGISTRY_LIMIT", message)
}
