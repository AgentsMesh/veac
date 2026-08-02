use std::collections::BTreeMap;

use crate::program::model::{ComponentDecl, ComponentKey, InstanceDecl, SurfaceFile};

use super::ENTRY_BYTES;

const COMPONENT_RECORDS: usize = 3;
const COMPONENT_PATH_COPIES: usize = 3;
const COMPONENT_NAME_COPIES: usize = 4;
const MEMBER_RECORDS: usize = 2;

pub(super) fn logical_bytes(
    file: &SurfaceFile,
    captured: &BTreeMap<String, ComponentKey>,
) -> Option<usize> {
    let mut bytes = file.source.len().checked_add(ENTRY_BYTES)?;
    for declaration in &file.components {
        bytes = add(bytes, declaration_bytes(&file.path, declaration)?)?;
    }
    for (alias, key) in captured {
        bytes = add(bytes, ENTRY_BYTES)?;
        bytes = add(bytes, alias.len())?;
        bytes = add(bytes, key.path.len())?;
        bytes = add(bytes, key.name.len())?;
    }
    Some(bytes)
}

fn declaration_bytes(path: &str, value: &ComponentDecl) -> Option<usize> {
    let mut bytes = ENTRY_BYTES.checked_mul(COMPONENT_RECORDS)?;
    bytes = add(bytes, path.len().checked_mul(COMPONENT_PATH_COPIES)?)?;
    bytes = add(bytes, value.name.len().checked_mul(COMPONENT_NAME_COPIES)?)?;
    for parameter in &value.parameters {
        bytes = add(bytes, ENTRY_BYTES.checked_mul(MEMBER_RECORDS)?)?;
        bytes = add(bytes, parameter.name.len().checked_mul(2)?)?;
        if let Some(default) = &parameter.default {
            bytes = add(bytes, default.source.len())?;
        }
    }
    for slot in &value.slots {
        bytes = add(bytes, ENTRY_BYTES.checked_mul(MEMBER_RECORDS)?)?;
        bytes = add(bytes, slot.name.len().checked_mul(2)?)?;
    }
    for instance in &value.instances {
        bytes = add(bytes, instance_bytes(instance)?)?;
    }
    Some(bytes)
}

fn instance_bytes(value: &InstanceDecl) -> Option<usize> {
    let mut bytes = ENTRY_BYTES.checked_add(value.component.len())?;
    bytes = add(bytes, value.id.len())?;
    for (name, binding) in &value.bindings {
        bytes = add(bytes, ENTRY_BYTES)?;
        bytes = add(bytes, name.len())?;
        bytes = add(bytes, binding.source.len())?;
    }
    for name in value.fills.keys() {
        bytes = add(bytes, ENTRY_BYTES)?;
        bytes = add(bytes, name.len())?;
    }
    Some(bytes)
}

fn add(left: usize, right: usize) -> Option<usize> {
    left.checked_add(right)
}

#[cfg(test)]
mod tests;
