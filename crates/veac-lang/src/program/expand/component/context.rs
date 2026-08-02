use std::collections::BTreeMap;

use crate::program::expression::ValueLookup;
use crate::program::model::{ComponentKey, PresetMap, SurfaceFile};

use super::super::hygiene::InstancePath;

pub(in crate::program::expand) struct Caller<'a> {
    pub path: &'a str,
    pub source: &'a str,
    pub values: &'a dyn ValueLookup,
    pub presets: &'a PresetMap,
    pub components: &'a BTreeMap<String, ComponentKey>,
    pub forwarded_slots: &'a BTreeMap<String, String>,
    pub hygiene_path: Option<InstancePath>,
}

impl<'a> Caller<'a> {
    pub(super) fn entry(
        file: &'a SurfaceFile,
        scope: super::super::scope::View<'a>,
        forwarded_slots: &'a BTreeMap<String, String>,
    ) -> Self {
        Self {
            path: &file.path,
            source: &file.source,
            values: scope.values,
            presets: scope.presets,
            components: scope.components,
            forwarded_slots,
            hygiene_path: None,
        }
    }
}
