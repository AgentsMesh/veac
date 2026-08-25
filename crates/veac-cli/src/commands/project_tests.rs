use std::path::Path;

use super::{authoring_error, project_package_error};
use veac_project::{
    IssueCode, ProjectAuthoringError, ProjectDecodeError, ProjectIssue, ProjectIssues,
};

fn code(error: crate::error::CliError) -> String {
    error.diagnostics()[0].code.clone()
}

#[test]
fn project_package_errors_keep_the_stable_contract_code() {
    let error = project_package_error(veac_build::BuildError::invalid("package drift"));
    assert_eq!(code(error), "PROJECT_PACKAGE_CONTRACT");
}

#[test]
fn authoring_load_and_decode_errors_preserve_repair_context() {
    let load = authoring_error(
        Path::new("project.veac"),
        ProjectAuthoringError::Load("missing source".to_owned()),
    );
    assert_eq!(code(load), "PROJECT_SOURCE_LOAD");

    let decode = authoring_error(
        Path::new("project.veac"),
        ProjectAuthoringError::Decode(ProjectDecodeError {
            path: "manifest.id".to_owned(),
            message: "invalid identifier".to_owned(),
        }),
    );
    assert_eq!(code(decode), "PROJECT_VALUE_DECODE");
    let validation = authoring_error(
        Path::new("project.veac"),
        ProjectAuthoringError::Validation(ProjectIssues(vec![ProjectIssue {
            code: IssueCode::InvalidId,
            path: "id".to_owned(),
            message: "invalid".to_owned(),
        }])),
    );
    assert_eq!(code(validation), "INVALID_ID");
}

#[test]
fn authoring_serialization_errors_are_mapped_without_panics() {
    let source = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    let error = authoring_error(
        Path::new("project.veac"),
        ProjectAuthoringError::Serialization(source),
    );
    assert_eq!(code(error), "PROJECT_SERIALIZATION");
}
