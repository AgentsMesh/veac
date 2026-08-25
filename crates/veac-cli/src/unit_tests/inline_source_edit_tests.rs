use tempfile::tempdir;

use super::support::{source_file, FakeEnvironment, EXECUTABLE_SOURCE};
use crate::Cli;

#[test]
fn source_edit_decodes_inline_values_from_the_candidate_declaration() {
    let temp = tempdir().unwrap();
    let source_text = format!("input parameter choice: int;\n{EXECUTABLE_SOURCE}");
    let source = source_file(&temp, &source_text);
    let prepared = veac_lang::program::prepare_path(&source).unwrap();
    let mut batch = veac_lang::source_edit::SourceEditBatch::new(
        veac_ir::OperationId::new("op_inline_candidate_type").unwrap(),
        prepared.source_revision().unwrap(),
    );
    batch.operations.push(
        veac_lang::source_edit::SourceEditOperation::SetDeclaration {
            target: veac_lang::source_edit::SourceNodeRef::input("main.veac", "choice"),
            site: veac_lang::source_edit::DeclarationSite::BuildInputDeclaration,
            declaration: veac_lang::source_edit::DeclarationSource {
                source: "input parameter choice: text;".into(),
            },
        },
    );
    let batch_path = temp.path().join("edit.json");
    std::fs::write(&batch_path, serde_json::to_vec(&batch).unwrap()).unwrap();
    let cli = Cli::try_parse_from([
        "veac",
        "source-edit",
        source.to_str().unwrap(),
        batch_path.to_str().unwrap(),
        "--input",
        "choice=hello=world",
        "--dry-run",
    ])
    .unwrap();
    crate::execute_with_environment(cli, &FakeEnvironment::success()).unwrap();
    assert_eq!(std::fs::read_to_string(source).unwrap(), source_text);
}

#[test]
fn source_edit_merges_manifest_and_inline_against_candidate_imported_enum() {
    let temp = tempdir().unwrap();
    std::fs::write(
        temp.path().join("locales.veac"),
        "module { export enum Locale { en, zh-Hans, } }",
    )
    .unwrap();
    let source_text = format!(
        "import \"./locales.veac\" as localization;\n\
         input parameter choice: int;\n{EXECUTABLE_SOURCE}"
    );
    let source = source_file(&temp, &source_text);
    let prepared = veac_lang::program::prepare_path(&source).unwrap();
    let mut batch = veac_lang::source_edit::SourceEditBatch::new(
        veac_ir::OperationId::new("op_inline_candidate_enum").unwrap(),
        prepared.source_revision().unwrap(),
    );
    batch.operations.push(
        veac_lang::source_edit::SourceEditOperation::SetDeclaration {
            target: veac_lang::source_edit::SourceNodeRef::input("main.veac", "choice"),
            site: veac_lang::source_edit::DeclarationSite::BuildInputDeclaration,
            declaration: veac_lang::source_edit::DeclarationSource {
                source: "input parameter choice: localization.Locale;".into(),
            },
        },
    );
    let batch_path = temp.path().join("edit.json");
    std::fs::write(&batch_path, serde_json::to_vec(&batch).unwrap()).unwrap();
    let manifest_path = temp.path().join("inputs.json");
    std::fs::write(
        &manifest_path,
        r#"{"schema":"https://veac.dev/schemas/build-inputs","schema_version":1,"inputs":[{"name":"choice","value":{"type":"enum","value":"en"}}]}"#,
    )
    .unwrap();
    let cli = Cli::try_parse_from([
        "veac",
        "source-edit",
        source.to_str().unwrap(),
        batch_path.to_str().unwrap(),
        "--inputs",
        manifest_path.to_str().unwrap(),
        "--input",
        "choice=zh-Hans",
        "--dry-run",
    ])
    .unwrap();
    crate::execute_with_environment(cli, &FakeEnvironment::success()).unwrap();
    assert_eq!(std::fs::read_to_string(source).unwrap(), source_text);
}
