use std::collections::BTreeMap;

use crate::program::model::{ComponentKey, PresetMap, Scope, ValueMap};

#[derive(Clone, Copy)]
pub(super) struct View<'a> {
    pub values: &'a ValueMap,
    pub presets: &'a PresetMap,
    pub components: &'a BTreeMap<String, ComponentKey>,
}

impl<'a> From<&'a Scope> for View<'a> {
    fn from(scope: &'a Scope) -> Self {
        Self {
            values: scope.values.as_ref(),
            presets: scope.presets.as_ref(),
            components: &scope.components,
        }
    }
}
