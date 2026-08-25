use std::path::{Path, PathBuf};

use veac_build::{ProjectAction, ProjectBuildPlan, ProjectBuildReceipt};
use veac_project::AuthoredProjectManifest;

use super::{build_error, paths::ProjectExecutionRoots};
use crate::error::{CliError, CliResult};

pub(super) fn write(
    receipt: &ProjectBuildReceipt,
    destination: Option<&Path>,
    project: &Path,
    project_root: &Path,
    authored: &AuthoredProjectManifest,
    plan: &ProjectBuildPlan,
    roots: &ProjectExecutionRoots,
) -> CliResult {
    let mut bytes = veac_build::canonical_project_receipt_bytes(receipt).map_err(build_error)?;
    bytes.push(b'\n');
    let json = String::from_utf8(bytes)
        .map_err(|error| CliError::new("PROJECT_RECEIPT_ENCODE", error.to_string()))?;
    let Some(destination) = destination else {
        return crate::fs::write_stdout(&json);
    };
    if destination == Path::new("-") {
        return crate::fs::write_stdout(&json);
    }
    let protected = protected_inputs(project, project_root, authored, plan, roots);
    let destination = destination_path(destination, &roots.build, &protected)?;
    crate::fs::atomic_write(&destination, &json)
}

fn protected_inputs(
    project: &Path,
    project_root: &Path,
    authored: &AuthoredProjectManifest,
    plan: &ProjectBuildPlan,
    roots: &ProjectExecutionRoots,
) -> Vec<PathBuf> {
    let mut protected = vec![project.to_path_buf()];
    protected.extend(authored.sources.keys().map(|path| project_root.join(path)));
    for id in plan.graph.topology() {
        let action = plan
            .graph
            .node(id)
            .expect("validated project node")
            .action();
        match action {
            ProjectAction::VeacRender {
                source,
                source_graph,
                ..
            } => {
                protected.push(roots.source.join(&source.path));
                protected.extend(render_source_graph_inputs(
                    &roots.source,
                    &source.path,
                    source_graph,
                ));
            }
            ProjectAction::Evidence {
                contract,
                source_graph,
                ..
            } => {
                protected.push(roots.source.join(&contract.path));
                protected.extend(root_source_graph_inputs(&roots.source, source_graph));
            }
            ProjectAction::MediaDerivation { .. } => {}
        }
        protected.extend(
            action
                .computation()
                .bound_sources
                .iter()
                .map(|source| roots.material.join(&source.snapshot.path)),
        );
    }
    protected.sort();
    protected.dedup();
    protected
}

fn render_source_graph_inputs(
    source_root: &Path,
    entry: &str,
    revision: &veac_build::ProjectSourceGraphRevision,
) -> Vec<PathBuf> {
    let entry = source_root.join(entry);
    let root = entry.parent().unwrap_or(source_root);
    revision
        .authored_modules
        .iter()
        .map(|module| root.join(module))
        .collect()
}

fn root_source_graph_inputs(
    source_root: &Path,
    revision: &veac_build::ProjectSourceGraphRevision,
) -> Vec<PathBuf> {
    revision
        .authored_modules
        .iter()
        .map(|module| source_root.join(module))
        .collect()
}

fn destination_path(
    destination: &Path,
    build_root: &Path,
    protected: &[PathBuf],
) -> CliResult<PathBuf> {
    let destination =
        crate::output::guarded_write_many(destination, protected.iter().map(PathBuf::as_path))?;
    if !destination.starts_with(build_root) {
        return Err(CliError::new(
            "PROJECT_RECEIPT_PATH",
            "project receipts must be written inside paths.build_root",
        ));
    }
    Ok(destination)
}

#[cfg(test)]
#[path = "receipt/tests.rs"]
mod tests;
