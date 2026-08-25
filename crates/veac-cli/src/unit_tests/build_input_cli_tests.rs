use tempfile::tempdir;

use super::support::{source_file, FakeEnvironment, EXECUTABLE_SOURCE};
use crate::Cli;

fn parse(args: &[&str]) -> Cli {
    Cli::try_parse_from(args).unwrap()
}

#[test]
fn build_and_check_accept_inputs_while_source_index_rejects_them() {
    let temp = tempdir().unwrap();
    let source_text = format!(
        "input parameter duration: time;\n{}",
        EXECUTABLE_SOURCE.replacen("200ms", "duration", 1)
    );
    let source = source_file(&temp, &source_text);
    let inputs = temp.path().join("inputs.json");
    std::fs::write(
        &inputs,
        r##"{
          "schema":"https://veac.dev/schemas/build-inputs",
          "schema_version":1,
          "inputs":[{"name":"duration","value":{"type":"time","value":"350ms"}}]
        }"##,
    )
    .unwrap();
    let output = temp.path().join("project.json");
    let environment = FakeEnvironment::success();

    for arguments in [
        vec![
            "veac",
            "build",
            source.to_str().unwrap(),
            "--inputs",
            inputs.to_str().unwrap(),
            "--input",
            "duration=400ms",
            "--emit-ir",
            output.to_str().unwrap(),
        ],
        vec![
            "veac",
            "check",
            source.to_str().unwrap(),
            "--inputs",
            inputs.to_str().unwrap(),
            "--input",
            "duration=400ms",
        ],
    ] {
        crate::execute_with_environment(parse(&arguments), &environment).unwrap();
    }
    assert!(Cli::try_parse_from([
        "veac",
        "source-index",
        source.to_str().unwrap(),
        "--inputs",
        inputs.to_str().unwrap(),
    ])
    .is_err());
    assert!(Cli::try_parse_from([
        "veac",
        "source-index",
        source.to_str().unwrap(),
        "--input",
        "duration=400ms",
    ])
    .is_err());
    let project = crate::canonical::load(&output).unwrap();
    assert_eq!(
        project.project.sequences[0].tracks[0].clips[0]
            .record_range
            .duration
            .value,
        400
    );
    let manifest_before = std::fs::read(&inputs).unwrap();
    let error = crate::execute(parse(&[
        "veac",
        "build",
        source.to_str().unwrap(),
        "--inputs",
        inputs.to_str().unwrap(),
        "--emit-ir",
        inputs.to_str().unwrap(),
    ]))
    .unwrap_err();
    assert!(error.to_string().contains("OUTPUT_OVERWRITES_INPUT"));
    assert_eq!(std::fs::read(&inputs).unwrap(), manifest_before);
}

#[test]
fn cli_rejects_missing_and_unknown_manifest_fields() {
    let temp = tempdir().unwrap();
    let source = source_file(
        &temp,
        &format!("input parameter duration: time;\n{EXECUTABLE_SOURCE}"),
    );
    let error = crate::execute(parse(&["veac", "check", source.to_str().unwrap()])).unwrap_err();
    assert!(error.to_string().contains("PROGRAM_INPUT_MISSING"));

    let inputs = temp.path().join("invalid.json");
    std::fs::write(
        &inputs,
        r#"{"schema":"https://veac.dev/schemas/build-inputs","schema_version":1,"unknown":true,"inputs":[]}"#,
    )
    .unwrap();
    let error = crate::execute(parse(&[
        "veac",
        "check",
        source.to_str().unwrap(),
        "--inputs",
        inputs.to_str().unwrap(),
    ]))
    .unwrap_err();
    assert!(error.to_string().contains("PROGRAM_INPUT_MANIFEST_JSON"));
}

#[test]
fn source_edit_rebuilds_with_the_explicit_input_manifest() {
    let temp = tempdir().unwrap();
    let source_text = format!(
        "input parameter duration: time;\n{}",
        EXECUTABLE_SOURCE.replacen("200ms", "duration", 1)
    );
    let source = source_file(&temp, &source_text);
    let inputs = temp.path().join("inputs.json");
    std::fs::write(
        &inputs,
        r#"{"schema":"https://veac.dev/schemas/build-inputs","schema_version":1,"inputs":[{"name":"duration","value":{"type":"time","value":"350ms"}}]}"#,
    )
    .unwrap();
    let prepared = veac_lang::program::prepare_path(&source).unwrap();
    let mut batch = veac_lang::source_edit::SourceEditBatch::new(
        veac_ir::OperationId::new("op_cli_input").unwrap(),
        prepared.source_revision().unwrap(),
    );
    batch.operations.push(
        veac_lang::source_edit::SourceEditOperation::SetDeclaration {
            target: veac_lang::source_edit::SourceNodeRef::input("main.veac", "duration"),
            site: veac_lang::source_edit::DeclarationSite::BuildInputDeclaration,
            declaration: veac_lang::source_edit::DeclarationSource {
                source: "input analysis duration: time;".into(),
            },
        },
    );
    let batch_path = temp.path().join("edit.json");
    std::fs::write(&batch_path, serde_json::to_vec(&batch).unwrap()).unwrap();
    let environment = FakeEnvironment::success();
    crate::execute_with_environment(
        parse(&[
            "veac",
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
            "--inputs",
            inputs.to_str().unwrap(),
            "--input",
            "duration=400ms",
            "--dry-run",
        ]),
        &environment,
    )
    .unwrap();
    let inputs_before = std::fs::read(&inputs).unwrap();
    let error = crate::execute(parse(&[
        "veac",
        "source-edit",
        source.to_str().unwrap(),
        batch_path.to_str().unwrap(),
        "--inputs",
        inputs.to_str().unwrap(),
        "--output",
        inputs.to_str().unwrap(),
    ]))
    .unwrap_err();
    assert!(error.to_string().contains("OUTPUT_OVERWRITES_INPUT"));
    assert_eq!(std::fs::read(&inputs).unwrap(), inputs_before);
    let error = crate::execute(parse(&[
        "veac",
        "source-edit",
        source.to_str().unwrap(),
        batch_path.to_str().unwrap(),
        "--dry-run",
    ]))
    .unwrap_err();
    assert!(error.to_string().contains("PROGRAM_INPUT_MISSING"));
    assert_eq!(std::fs::read_to_string(source).unwrap(), source_text);
}
