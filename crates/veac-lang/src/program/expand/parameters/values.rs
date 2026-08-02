use std::collections::BTreeMap;
use std::sync::Arc;

use crate::program::diagnostic::Diagnostic;
use crate::program::expression::{Value, ValueLookup};
use crate::program::model::{ComponentInterface, ValueMap};

use super::super::budget;

const ENTRY_BYTES: usize = 64;

pub(in crate::program::expand) struct BoundValues<'a> {
    base: &'a ValueMap,
    interface: &'a ComponentInterface,
    local: BTreeMap<String, Arc<Value>>,
    retained_bytes: usize,
}

impl<'a> BoundValues<'a> {
    pub(super) fn new(base: &'a ValueMap, interface: &'a ComponentInterface) -> Self {
        Self {
            base,
            interface,
            local: BTreeMap::new(),
            retained_bytes: 0,
        }
    }

    pub(super) fn insert(
        &mut self,
        path: &str,
        name: &str,
        value: Value,
        limit: usize,
    ) -> Result<(), Diagnostic> {
        let added = budget::checked_source_add(path, ENTRY_BYTES, name.len())?;
        let added = budget::checked_source_add(path, added, value.retained_bytes())?;
        let retained = budget::checked_source_add(path, self.retained_bytes, added)?;
        budget::ensure_source(path, retained, limit)?;
        self.local.insert(name.to_owned(), Arc::new(value));
        self.retained_bytes = retained;
        Ok(())
    }

    pub(super) fn is_resolved(&self, name: &str) -> bool {
        self.local.contains_key(name)
    }

    pub(in crate::program::expand) fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
}

#[cfg(test)]
mod tests;

impl ValueLookup for BoundValues<'_> {
    fn value(&self, name: &str) -> Option<&Value> {
        self.local.get(name).map(Arc::as_ref).or_else(|| {
            (!self.interface.has_parameter(name))
                .then(|| self.base.value(name))
                .flatten()
        })
    }
}
