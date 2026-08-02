use super::support::*;
use veac_lang::source_edit::{
    ExpressionSite, ExpressionSource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
    SourceRevision,
};

fn program_source() -> String {
    format!(
        "const time duration = 200ms;\n{}",
        GENERATED_SOURCE.replace("duration 200ms", "duration ${duration}")
    )
}

fn revision(source: &std::path::Path) -> SourceRevision {
    let output = veac()
        .args(["source-revision", source.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn batch(revision: SourceRevision, expression: &str) -> SourceEditBatch {
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_cli_source_edit").unwrap(),
        revision,
    );
    batch.operations.push(SourceEditOperation::SetExpression {
        target: SourceNodeRef::constant("main.veac", "duration"),
        site: ExpressionSite::ConstantValue,
        expression: ExpressionSource {
            source: expression.into(),
        },
    });
    batch
}

#[test]
fn source_revision_and_source_edit_form_a_validated_source_of_truth_loop() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &program_source());
    let base = revision(&source);
    let batch = batch(base.clone(), "400ms");
    let batch_path = temp.path().join("source-edit.json");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&batch).unwrap(),
    )
    .unwrap();

    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dry_run\": true"));
    assert!(std::fs::read_to_string(&source).unwrap().contains("200ms"));
    assert!(!temp.path().join(".veac-source.lock").exists());

    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"module\": \"main.veac\""));
    let edited = std::fs::read_to_string(&source).unwrap();
    assert!(edited.starts_with("const time duration = 400ms;"));
    assert!(edited.contains("duration ${duration}"));
    assert_ne!(revision(&source), base);

    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("stale source revision"));
}

#[test]
fn source_edit_output_copy_preserves_the_original_source() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &program_source());
    let batch_path = temp.path().join("source-edit.json");
    let output = temp.path().join("edited.veac");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&batch(
            revision(&source),
            "500ms",
        ))
        .unwrap(),
    )
    .unwrap();

    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
            "--out",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert!(std::fs::read_to_string(source).unwrap().contains("200ms"));
    assert!(std::fs::read_to_string(output).unwrap().contains("500ms"));
    assert!(!temp.path().join(".veac-source.lock").exists());
}

#[test]
fn rejected_source_edit_never_changes_source_or_overwrites_its_batch() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &program_source());
    let original = std::fs::read_to_string(&source).unwrap();
    let batch_path = temp.path().join("source-edit.json");
    let invalid = batch(revision(&source), "20px");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&invalid).unwrap(),
    )
    .unwrap();

    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("PROGRAM_CONST_TYPE"));
    assert_eq!(std::fs::read_to_string(&source).unwrap(), original);

    let valid_json = veac_lang::source_edit::canonical_source_edit_batch_json(&batch(
        revision(&source),
        "600ms",
    ))
    .unwrap();
    std::fs::write(&batch_path, &valid_json).unwrap();
    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
            "--output",
            batch_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("OUTPUT_OVERWRITES_INPUT"));
    assert_eq!(std::fs::read_to_string(batch_path).unwrap(), valid_json);
    assert_eq!(std::fs::read_to_string(source).unwrap(), original);
}

#[test]
fn formatter_recognizes_program_syntax_without_line_prefix_heuristics() {
    let temp = tempdir().unwrap();
    let source = source_file(
        &temp,
        &program_source().replacen("const time", "/* typed declaration */ const\ntime", 1),
    );
    let original = std::fs::read_to_string(&source).unwrap();

    veac()
        .args(["fmt", source.to_str().unwrap(), "--check"])
        .assert()
        .success();
    assert_eq!(std::fs::read_to_string(source).unwrap(), original);
}

#[path = "source_program/compile_output.rs"]
mod compile_output;
#[path = "source_program/errors.rs"]
mod errors;
#[path = "source_program/module_edit.rs"]
mod module_edit;
#[path = "source_program/read_only.rs"]
mod read_only;
#[path = "source_program/security.rs"]
mod security;
