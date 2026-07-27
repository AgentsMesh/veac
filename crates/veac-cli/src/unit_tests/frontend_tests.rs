use tempfile::tempdir;

use super::support::{source_file, GENERATED_SOURCE};

#[test]
fn frontend_compiles_revision_and_formats_authoring_source() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, GENERATED_SOURCE);
    let project = crate::frontend::compile(&source, 42).unwrap();
    assert_eq!(project.project.revision, 42);
    assert_eq!(project.project.id.as_str(), "prj_cli-test");

    let compact = GENERATED_SOURCE.replace("  ", " ");
    std::fs::write(&source, compact).unwrap();
    let (before, after) = crate::frontend::format(&source).unwrap();
    assert_ne!(before, after);
    assert!(after.starts_with("project cli-test {\n"));
}

#[test]
fn authoring_errors_include_source_locations() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, "project broken {\n  nope;\n  also-nope;\n}");
    let error = crate::frontend::compile(&source, 0).unwrap_err();
    let rendered = error.to_string();
    assert!(rendered.contains("error[AUTHORING_"));
    assert!(rendered.contains("main.veac:"));
    assert!(error.diagnostics().len() >= 2);

    let json: serde_json::Value = serde_json::from_str(&error.diagnostics_json().unwrap()).unwrap();
    assert!(json["diagnostics"].as_array().unwrap().len() >= 2);
    assert!(json["diagnostics"][0]["source_span"]["path"]
        .as_str()
        .unwrap()
        .ends_with("main.veac"));
}

#[test]
fn semantic_lowering_errors_are_typed() {
    let temp = tempdir().unwrap();
    let invalid = GENERATED_SOURCE.replace(
        "record { at 0s; duration 200ms; }",
        "record { at 0s; duration 200ms; }\n        mapping linear { from 0s; to 200ms; }",
    );
    let source = source_file(&temp, &invalid);

    let error = crate::frontend::compile(&source, 0).unwrap_err();
    assert!(error.to_string().contains("AUTHORING_LOWER_MAPPING_SOURCE"));
}

#[test]
fn source_io_and_formatter_errors_are_typed() {
    let temp = tempdir().unwrap();
    let missing = temp.path().join("missing.veac");
    assert!(crate::frontend::compile(&missing, 0)
        .unwrap_err()
        .to_string()
        .contains("READ_FAILED"));
    assert!(crate::frontend::format(temp.path())
        .unwrap_err()
        .to_string()
        .contains("READ_FAILED"));

    let invalid = source_file(&temp, "project x { settings { timebase 1/1000; }");
    assert!(crate::frontend::format(&invalid)
        .unwrap_err()
        .to_string()
        .contains("AUTHORING_"));
}

#[test]
fn cli_error_is_a_standard_error() {
    let error = crate::CliError::new("TEST", "message");
    assert_eq!(
        error.to_string(),
        "error[TEST]: message\n  help: correct the reported condition and retry"
    );
    let as_error: &dyn std::error::Error = &error;
    assert!(as_error.source().is_none());
}
