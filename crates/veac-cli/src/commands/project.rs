use std::path::{Path, PathBuf};

use veac_project::{AuthoredProjectManifest, ProjectAuthoringError, ProjectIssues};

use crate::arguments::ProjectCommand;
use crate::diagnostic::CliDiagnostic;
use crate::error::{CliError, CliResult};

mod execution;

pub(crate) fn run(command: ProjectCommand) -> CliResult {
    match command {
        ProjectCommand::Check {
            project,
            package_roots,
        } => check(&project, &package_roots),
        ProjectCommand::Inspect {
            project,
            package_roots,
        } => inspect(&project, &package_roots),
        ProjectCommand::Graph {
            project,
            package_roots,
        } => graph(&project, &package_roots),
        ProjectCommand::Build {
            project,
            receipt,
            package_roots,
        } => execution::run(
            &project,
            receipt.as_deref(),
            &package_roots,
            execution::ExecutionMode::Build,
        ),
        ProjectCommand::Evidence {
            project,
            receipt,
            package_roots,
        } => execution::run(
            &project,
            receipt.as_deref(),
            &package_roots,
            execution::ExecutionMode::Evidence,
        ),
        ProjectCommand::Test {
            project,
            receipt,
            package_roots,
        } => execution::run(
            &project,
            receipt.as_deref(),
            &package_roots,
            execution::ExecutionMode::Test,
        ),
    }
}

fn check(path: &Path, package_roots: &[PathBuf]) -> CliResult {
    let (_, project, graph, _) = load(path, package_roots)?;
    crate::fs::write_stdout(&format!(
        "Project is valid: {} (project {}, {} target instance(s), manifest {})\n",
        path.display(),
        project.manifest.id,
        graph.instances.len(),
        project.manifest_digest
    ))
}

fn inspect(path: &Path, package_roots: &[PathBuf]) -> CliResult {
    let (_, project, _, _) = load(path, package_roots)?;
    let mut json = match veac_project::canonical_manifest_json(&project.manifest) {
        Ok(json) => json,
        Err(error) => return Err(CliError::new("PROJECT_SERIALIZATION", error.to_string())),
    };
    json.push('\n');
    crate::fs::write_stdout(&json)
}

fn graph(path: &Path, package_roots: &[PathBuf]) -> CliResult {
    let (_, _, graph, _) = load(path, package_roots)?;
    let mut json = match serde_json_canonicalizer::to_string(&graph) {
        Ok(json) => json,
        Err(error) => return Err(CliError::new("PROJECT_SERIALIZATION", error.to_string())),
    };
    json.push('\n');
    crate::fs::write_stdout(&json)
}

fn load(
    path: &Path,
    package_roots: &[PathBuf],
) -> CliResult<(
    PathBuf,
    AuthoredProjectManifest,
    veac_project::ResolvedTargetGraph,
    veac_build::ProjectPackageSet,
)> {
    let packages =
        veac_build::ProjectPackageSet::capture(package_roots).map_err(project_package_error)?;
    packages.revalidate().map_err(project_package_error)?;
    let (project_loader, entry) = veac_lang::program::FileSystemLoader::for_entry(path)
        .map_err(|message| CliError::new("PROJECT_SOURCE_LOAD", message))?;
    let root = project_loader.root().to_owned();
    let loader = packages
        .loader(project_loader)
        .map_err(project_package_error)?;
    let project = veac_project::build_project_with_loader(entry, &loader)
        .map_err(|error| authoring_error(path, error))?;
    packages.revalidate().map_err(project_package_error)?;
    let graph = veac_project::resolve_manifest(&project.manifest).map_err(project_issues)?;
    execution::validate_package_authorities(&root, &project.manifest.paths, &packages)?;
    Ok((root, project, graph, packages))
}

fn project_package_error(error: veac_build::BuildError) -> CliError {
    CliError::new("PROJECT_PACKAGE_CONTRACT", error.to_string())
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

#[cfg(test)]
#[path = "project_tests.rs"]
mod tests;
