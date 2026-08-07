use std::fs;

use tempfile::tempdir;
use veac_lang::program::{
    apply_executable_source_edit_path_with_inputs, prepare_path, BuildInputBinding,
    BuildInputManifestV1, BuildInputManifestValue, SourceTransactionError,
};
use veac_lang::source_edit::{
    DeclarationSite, DeclarationSource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

#[path = "program_functions/support.rs"]
mod support;

fn manifest() -> BuildInputManifestV1 {
    let mut manifest = BuildInputManifestV1::empty();
    manifest.inputs.push(BuildInputBinding {
        name: "duration".into(),
        value: BuildInputManifestValue::Time {
            value: "1750ms".into(),
        },
    });
    manifest
}

#[test]
fn structural_input_edit_rebuilds_with_the_same_verified_manifest() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let source = support::project_with("input parameter duration: time;", "duration");
    fs::write(&entry, &source).unwrap();
    let prepared = prepare_path(&entry).unwrap();
    let baseline = prepared.execute_with_inputs(&manifest()).unwrap();
    let index = prepared.source_index().unwrap();
    let target = SourceNodeRef::input("main.veac", "duration");
    assert!(index
        .declaration(&target, DeclarationSite::BuildInputDeclaration)
        .is_some());

    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_input_role").unwrap(),
        index.revision().clone(),
    );
    batch.operations.push(SourceEditOperation::SetDeclaration {
        target,
        site: DeclarationSite::BuildInputDeclaration,
        declaration: DeclarationSource {
            source: "input analysis duration: time;".into(),
        },
    });
    let preview =
        apply_executable_source_edit_path_with_inputs(&entry, &batch, &manifest()).unwrap();
    assert!(preview
        .source()
        .unwrap()
        .contains("input analysis duration: time;"));
    assert_eq!(support::result_duration(&preview.built), "1750ms");
    assert_ne!(
        baseline
            .envelope()
            .executable
            .digests
            .declared_inputs_sha256,
        preview
            .built
            .envelope()
            .executable
            .digests
            .declared_inputs_sha256
    );
    assert_eq!(fs::read_to_string(entry).unwrap(), source);
}

#[test]
fn source_edit_does_not_fall_back_to_ambient_or_empty_inputs() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(
        &entry,
        support::project_with("input parameter duration: time;", "duration"),
    )
    .unwrap();
    let prepared = prepare_path(&entry).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_missing_input").unwrap(),
        prepared.source_index().unwrap().revision().clone(),
    );
    batch.operations.push(SourceEditOperation::SetDeclaration {
        target: SourceNodeRef::input("main.veac", "duration"),
        site: DeclarationSite::BuildInputDeclaration,
        declaration: DeclarationSource {
            source: "input parameter duration: time;".into(),
        },
    });
    let error = apply_executable_source_edit_path_with_inputs(
        &entry,
        &batch,
        &BuildInputManifestV1::empty(),
    )
    .unwrap_err();
    let SourceTransactionError::Program(diagnostics) = error else {
        panic!("missing input must remain a typed program diagnostic");
    };
    assert_eq!(diagnostics.as_slice()[0].code, "PROGRAM_INPUT_MISSING");
}
