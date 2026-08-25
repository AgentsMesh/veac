use tempfile::tempdir;

use super::support::{source_file, EXECUTABLE_SOURCE};

#[test]
fn executable_frontend_builds_revision_and_returns_canonical_source() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, EXECUTABLE_SOURCE);
    let project = crate::frontend::check(&source, None, &[], &[], 42).unwrap();
    assert_eq!(project.project.revision, 42);
    assert!(project.project.id.as_str().starts_with("prj_"));

    let before = std::fs::read_to_string(&source).unwrap();
    let (read, formatted) = crate::frontend::format(&source, &[]).unwrap();
    assert_eq!(read, before);
    assert_ne!(formatted, before);
    assert_eq!(
        veac_lang::program::format_source(&formatted).unwrap(),
        formatted
    );
}

#[test]
fn executable_runtime_errors_include_source_locations() {
    let temp = tempdir().unwrap();
    let runtime = format!(
        "fn fail(value: int) -> int {{ let ignored = 1 / value; 10 }}\n{}",
        EXECUTABLE_SOURCE.replacen("frame_rate(10, 1)", "frame_rate(fail(0), 1)", 1)
    );
    let source = source_file(&temp, &runtime);
    let error = crate::frontend::check(&source, None, &[], &[], 0).unwrap_err();
    assert!(error.to_string().contains("PROGRAM_EXECUTABLE_RUNTIME"));
    assert!(error.to_string().contains("main.veac:"));
    let json: serde_json::Value = serde_json::from_str(&error.diagnostics_json().unwrap()).unwrap();
    assert_eq!(json["diagnostics"][0]["code"], "PROGRAM_EXECUTABLE_RUNTIME");
}

#[test]
fn source_io_and_executable_syntax_errors_are_typed() {
    let temp = tempdir().unwrap();
    let missing = temp.path().join("missing.veac");
    assert!(crate::frontend::check(&missing, None, &[], &[], 0)
        .unwrap_err()
        .to_string()
        .contains("PATH_UNAVAILABLE"));
    assert!(crate::frontend::format(temp.path(), &[])
        .unwrap_err()
        .to_string()
        .contains("INVALID_SOURCE_PATH"));

    let invalid = source_file(&temp, "fn main(context: Context) -> Project {");
    assert!(crate::frontend::format(&invalid, &[])
        .unwrap_err()
        .to_string()
        .contains("PROGRAM_"));
}

#[test]
fn source_preparation_and_entry_formatting_map_semantic_diagnostics() {
    let temp = tempdir().unwrap();
    let invalid = source_file(&temp, "fn main(context: Context) -> Project { 1 }\n");
    let prepared = crate::frontend::prepare_source_graph(&invalid, &[]).unwrap_err();
    assert!(prepared.to_string().contains("PROGRAM_"));
    let formatted = crate::frontend::format(&invalid, &[]).unwrap_err();
    assert!(formatted.to_string().contains("PROGRAM_"));
}

#[test]
fn cli_error_is_a_standard_error() {
    let error = crate::CliError::new("TEST", "message");
    assert!(!error.uses_json_diagnostics());
    assert_eq!(
        error.to_string(),
        "error[TEST]: message\n  help: correct the reported condition and retry"
    );
    let as_error: &dyn std::error::Error = &error;
    assert!(as_error.source().is_none());

    let json = crate::CliError::new("TEST", "message")
        .with_diagnostic_format(crate::DiagnosticFormat::Json);
    assert!(json.uses_json_diagnostics());
}

#[test]
fn formatter_canonicalizes_executable_modules_and_entries() {
    let temp = tempdir().unwrap();
    let module = source_file(
        &temp,
        "module { export fn rate(value: int) -> int { value } }",
    );
    let module_source = std::fs::read_to_string(&module).unwrap();
    let formatted_module = crate::frontend::format(&module, &[]).unwrap().1;
    assert_ne!(formatted_module, module_source);
    assert_eq!(
        formatted_module,
        "module {\n  export fn rate(value: int) -> int {\n    value\n  }\n}\n"
    );

    let entry = source_file(&temp, EXECUTABLE_SOURCE);
    let entry_source = std::fs::read_to_string(&entry).unwrap();
    let formatted_entry = crate::frontend::format(&entry, &[]).unwrap().1;
    assert_ne!(formatted_entry, entry_source);
    assert!(formatted_entry.starts_with("fn main(context: Context) -> Project {\n"));
}
