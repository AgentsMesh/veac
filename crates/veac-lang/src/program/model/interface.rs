use std::collections::BTreeMap;

use super::{ComponentDecl, ParameterDecl, SlotDecl};

#[derive(Debug, Clone)]
pub(crate) struct ComponentInterface {
    parameters: BTreeMap<String, usize>,
    slots: BTreeMap<String, usize>,
}

impl ComponentInterface {
    pub(crate) fn from_declaration(declaration: &ComponentDecl) -> Self {
        Self::from_members(&declaration.parameters, &declaration.slots)
    }

    pub(crate) fn from_members(parameters: &[ParameterDecl], slots: &[SlotDecl]) -> Self {
        Self {
            parameters: parameters
                .iter()
                .enumerate()
                .map(|(index, value)| (value.name.clone(), index))
                .collect(),
            slots: slots
                .iter()
                .enumerate()
                .map(|(index, value)| (value.name.clone(), index))
                .collect(),
        }
    }

    pub(crate) fn parameter(&self, name: &str) -> Option<usize> {
        self.parameters.get(name).copied()
    }

    pub(crate) fn has_parameter(&self, name: &str) -> bool {
        self.parameters.contains_key(name)
    }

    pub(crate) fn parameter_count(&self) -> usize {
        self.parameters.len()
    }

    pub(crate) fn has_slot(&self, name: &str) -> bool {
        self.slots.contains_key(name)
    }
}
