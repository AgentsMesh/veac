use std::collections::BTreeMap;

use super::*;
use crate::program::expression::{PrimitiveType, ValueType};

#[path = "tests/bindings.rs"]
mod binding_tests;
#[path = "tests/enums.rs"]
mod enum_tests;
#[path = "tests/manifest.rs"]
mod manifest_tests;
#[path = "tests/values.rs"]
mod value_tests;

fn declaration(name: &str, primitive: PrimitiveType) -> BuildInputDeclaration {
    BuildInputDeclaration::new(
        "entry.veac",
        name.to_owned(),
        BuildInputRole::Parameter,
        ValueType::from(primitive),
    )
}

fn declarations(values: &[(&str, PrimitiveType)]) -> BTreeMap<String, BuildInputDeclaration> {
    values
        .iter()
        .map(|(name, primitive)| ((*name).to_owned(), declaration(name, *primitive)))
        .collect()
}

fn binding(name: &str, value: BuildInputManifestValue) -> BuildInputBinding {
    BuildInputBinding {
        name: name.to_owned(),
        value,
    }
}

fn manifest(inputs: Vec<BuildInputBinding>) -> BuildInputManifestV1 {
    BuildInputManifestV1 {
        inputs,
        ..BuildInputManifestV1::empty()
    }
}
