use std::collections::BTreeMap;
use std::sync::Arc;

use super::PresetKind;

pub(crate) type PresetKey = (String, PresetKind);
pub(crate) type PresetMap = BTreeMap<PresetKey, Arc<str>>;

pub(crate) fn preset_key(kind: PresetKind, name: impl Into<String>) -> PresetKey {
    (name.into(), kind)
}

pub(crate) fn preset_name_exists<V>(values: &BTreeMap<PresetKey, V>, name: &str) -> bool {
    let first = preset_key(PresetKind::TextStyle, name);
    let last = preset_key(PresetKind::DeliveryProfile, name);
    values.range(first..=last).next().is_some()
}

#[cfg(test)]
#[path = "preset_map/tests.rs"]
mod tests;
