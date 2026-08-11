use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use veac_codegen::emitter::{
    BackendCommand, BackendFilterBinding, BackendFilterContract, BackendFilterEscape,
    BackendInternalAccess, BackendPreparation,
};

use super::{internal_produces, validate_binding, validate_command};

const TOKEN: &str = "__VEAC_FILTER_RESOURCE_0000__";

#[test]
fn preparation_outputs_must_be_present_unique_and_exactly_produced() {
    let mut command = base_command();
    command.preparations.push(BackendPreparation {
        command: base_command(),
        outputs: vec![],
    });
    rejects_command(&command, "at least one sidecar output");

    let mut command = base_command();
    command.preparations.push(BackendPreparation {
        command: base_command(),
        outputs: vec![PathBuf::from("sidecar"), PathBuf::from("sidecar")],
    });
    rejects_command(&command, "must be unique");

    let mut command = base_command();
    command.preparations.push(BackendPreparation {
        command: base_command(),
        outputs: vec![PathBuf::from("sidecar")],
    });
    rejects_command(&command, "exactly one internal producer");
}

#[test]
fn internal_bindings_enforce_producer_consumer_boundaries() {
    let sidecar = PathBuf::from("sidecar");
    let producer = internal(&sidecar, BackendInternalAccess::Produce);
    let consumer = internal(&sidecar, BackendInternalAccess::Consume);
    let empty = BTreeSet::new();

    let declared = [PathBuf::from("different")];
    assert!(validate_binding(&producer, &empty, &empty, Some(&declared)).is_err());
    assert!(validate_binding(&producer, &empty, &empty, None).is_err());
    assert!(validate_binding(&consumer, &empty, &empty, None).is_err());

    let external = BackendFilterBinding::file(
        TOKEN.to_owned(),
        PathBuf::from("unprotected.ttf"),
        BackendFilterEscape::Quoted,
    );
    assert!(validate_binding(&external, &empty, &empty, None).is_err());

    let mut command = base_command();
    command.filter_contract =
        Some(BackendFilterContract::new(TOKEN.to_owned(), vec![producer]).unwrap());
    assert_eq!(internal_produces(&command), BTreeMap::from([(sidecar, 1)]));

    let mut no_producer = base_command();
    no_producer.filter_contract =
        Some(BackendFilterContract::new(TOKEN.to_owned(), vec![consumer]).unwrap());
    assert!(internal_produces(&no_producer).is_empty());
}

#[test]
fn internal_bindings_reject_empty_or_traversing_paths() {
    for path in [
        Path::new(""),
        Path::new("../sidecar"),
        Path::new("/sidecar"),
    ] {
        let binding = internal(path, BackendInternalAccess::Consume);
        assert!(validate_binding(&binding, &BTreeSet::new(), &BTreeSet::new(), None).is_err());
    }
}

fn base_command() -> BackendCommand {
    BackendCommand {
        preparations: vec![],
        inputs: vec![],
        filter_graph: None,
        filter_contract: None,
        maps: vec![],
        output_args: vec![],
        output_path: "ignored.null".into(),
    }
}

fn internal(path: &Path, access: BackendInternalAccess) -> BackendFilterBinding {
    BackendFilterBinding::internal_file(
        TOKEN.to_owned(),
        path.to_path_buf(),
        access,
        BackendFilterEscape::Quoted,
    )
}

fn rejects_command(command: &BackendCommand, expected: &str) {
    let error = validate_command(command, &BTreeSet::new()).unwrap_err();
    assert!(error.message.contains(expected));
}
