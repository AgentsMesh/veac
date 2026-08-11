use std::path::Path;

use tempfile::tempdir;

use super::{EntryContract, EntryParameterContract, EntryValueType};
use crate::program::expression::{PrimitiveType, Value};
use crate::program::{prepare_host_path, prepare_host_root_path};

fn contract() -> EntryContract {
    EntryContract::new(
        "workspace",
        EntryValueType::Primitive(PrimitiveType::Integer),
    )
}

#[test]
fn filesystem_host_entries_preserve_canonical_roots_and_public_views() {
    let directory = tempdir().unwrap();
    std::fs::create_dir(directory.path().join("nested")).unwrap();
    let entry = directory.path().join("nested/main.veac");
    std::fs::write(&entry, "fn workspace() -> int { 7 }").unwrap();

    let (root, prepared) = prepare_host_path(&entry, &contract()).unwrap();
    assert_eq!(
        root,
        directory.path().join("nested").canonicalize().unwrap()
    );
    assert_eq!(prepared.root_module(), "main.veac");
    assert_eq!(prepared.sources().len(), 1);
    assert_eq!(prepared.type_registry().len(), 0);
    let evaluated = prepared.execute(&[]).unwrap();
    assert_eq!(evaluated.value(), &Value::Integer(7));
    assert_eq!(evaluated.type_registry().len(), 0);

    let rooted =
        prepare_host_root_path(directory.path(), Path::new("nested/main.veac"), &contract())
            .unwrap();
    assert_eq!(rooted.root_module(), "nested/main.veac");
}

#[test]
fn filesystem_host_load_errors_keep_the_requested_path() {
    let directory = tempdir().unwrap();
    let missing = directory.path().join("missing.veac");
    let error = prepare_host_path(&missing, &contract()).unwrap_err();
    let diagnostic = &error.as_slice()[0];
    assert_eq!(diagnostic.code, "PROGRAM_ENTRY_LOAD");
    assert_eq!(diagnostic.path, missing.display().to_string());
}

#[test]
fn entry_contract_accessors_publish_the_exact_host_abi() {
    let parameter =
        EntryParameterContract::new("value", EntryValueType::Primitive(PrimitiveType::Integer));
    assert_eq!(parameter.name(), "value");
    assert_eq!(
        parameter.value_type(),
        &EntryValueType::Primitive(PrimitiveType::Integer)
    );
    let contract = EntryContract::new("inspect", EntryValueType::nominal("Manifest"))
        .with_parameter(parameter)
        .with_prelude("abi.veac");
    assert_eq!(contract.function(), "inspect");
    assert_eq!(contract.parameters().len(), 1);
    assert_eq!(contract.result(), &EntryValueType::nominal("Manifest"));
    assert_eq!(contract.preludes(), &["abi.veac"]);
}
