use std::path::Path;

use veac_project::{AuthoredProjectManifest, ProjectAuthoringError, ProjectIssues};

use crate::arguments::ProjectCommand;
use crate::diagnostic::CliDiagnostic;
use crate::error::{CliError, CliResult};

mod execution;

pub(crate) fn run(command: ProjectCommand) -> CliResult {
    match command {
        ProjectCommand::Check { project } => check(&project),
        ProjectCommand::Inspect { project } => inspect(&project),
        ProjectCommand::Graph { project } => graph(&project),
        ProjectCommand::Build { project, receipt } => execution::run(
            &project,
            receipt.as_deref(),
            execution::ExecutionMode::Build,
        ),
        ProjectCommand::Evidence { project, receipt } => execution::run(
            &project,
            receipt.as_deref(),
            execution::ExecutionMode::Evidence,
        ),
        ProjectCommand::Test { project, receipt } => {
            execution::run(&project, receipt.as_deref(), execution::ExecutionMode::Test)
        }
    }
}

fn check(path: &Path) -> CliResult {
    let (project, graph) = load(path)?;
    crate::fs::write_stdout(&format!(
        "Project is valid: {} (project {}, {} target instance(s), manifest {})\n",
        path.display(),
        project.manifest.id,
        graph.instances.len(),
        project.manifest_digest
    ))
}

fn inspect(path: &Path) -> CliResult {
    let (project, _) = load(path)?;
    let mut json = match veac_project::canonical_manifest_json(&project.manifest) {
        Ok(json) => json,
        Err(error) => return Err(CliError::new("PROJECT_SERIALIZATION", error.to_string())),
    };
    json.push('\n');
    crate::fs::write_stdout(&json)
}

fn graph(path: &Path) -> CliResult {
    let (_, graph) = load(path)?;
    let mut json = match serde_json_canonicalizer::to_string(&graph) {
        Ok(json) => json,
        Err(error) => return Err(CliError::new("PROJECT_SERIALIZATION", error.to_string())),
    };
    json.push('\n');
    crate::fs::write_stdout(&json)
}

fn load(path: &Path) -> CliResult<(AuthoredProjectManifest, veac_project::ResolvedTargetGraph)> {
    let (_, project) = match veac_project::build_project_path(path) {
        Ok(project) => project,
        Err(error) => return Err(authoring_error(path, error)),
    };
    let graph = veac_project::resolve_manifest(&project.manifest).map_err(project_issues)?;
    Ok((project, graph))
}

fn authoring_error(path: &Path, error: ProjectAuthoringError) -> CliError {
    match error {
        ProjectAuthoringError::Load(message) => CliError::new("PROJECT_SOURCE_LOAD", message),
        ProjectAuthoringError::Language(errors) => crate::diagnostic::program(path, errors),
        ProjectAuthoringError::Decode(error) => CliError::from_diagnostics(vec![CliDiagnostic {
            code: "PROJECT_VALUE_DECODE".to_owned(),
            object_id: None,
            source_span: None,
            pointer: Some(error.path),
            location: None,
            message: error.message,
            suggested_repair: Some("return a value matching ProjectManifest".to_owned()),
        }]),
        ProjectAuthoringError::Validation(errors) => project_issues(errors),
        ProjectAuthoringError::Serialization(error) => {
            CliError::new("PROJECT_SERIALIZATION", error.to_string())
        }
    }
}

fn project_issues(errors: ProjectIssues) -> CliError {
    let diagnostics = errors
        .issues()
        .iter()
        .map(|issue| CliDiagnostic {
            code: issue_code(issue.code),
            object_id: None,
            source_span: None,
            pointer: Some(issue.path.clone()),
            location: None,
            message: issue.message.clone(),
            suggested_repair: Some("correct the project declaration and retry".to_owned()),
        })
        .collect();
    CliError::from_diagnostics(diagnostics)
}

fn issue_code(code: veac_project::IssueCode) -> String {
    serde_json::to_value(code)
        .expect("IssueCode serialization is infallible")
        .as_str()
        .expect("IssueCode uses a string representation")
        .to_owned()
}
