use std::path::{Path, PathBuf};

use veac_artifact::ArtifactStore;
use veac_build::{
    CancellationToken, ProjectBuildOutcome, ProjectBuildRuntime, ProjectGraphAdapter,
};

use crate::error::{CliError, CliResult};

mod backend;
mod evidence_result;
mod limits;
mod paths;
mod receipt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ExecutionMode {
    Build,
    Evidence,
    Test,
}

pub(super) fn run(
    project_path: &Path,
    receipt_path: Option<&Path>,
    package_roots: &[PathBuf],
    mode: ExecutionMode,
) -> CliResult {
    let (root, authored, graph, packages) = super::load(project_path, package_roots)?;
    let roots = paths::ProjectExecutionRoots::resolve(&root, &authored.manifest.paths)?;
    roots.validate_package_roots(&packages)?;
    let plan = ProjectGraphAdapter::new(&roots.source, &roots.material, packages.clone())
        .map_err(build_error)?
        .adapt(&graph)
        .map_err(build_error)?;
    let store = ArtifactStore::new(roots.cache.join("artifacts"));
    let backend = backend::CliProjectBackend::new(
        roots.source.clone(),
        roots.material.clone(),
        packages.clone(),
        store.clone(),
        plan.manifest_digest.clone(),
    );
    let runtime = ProjectBuildRuntime::new(
        limits::from_manifest(&authored.manifest)?,
        store.clone(),
        roots.cache.join("leases"),
        roots.build.join("staging"),
        roots.delivery.clone(),
        backend,
    )
    .map_err(build_error)?;
    let receipt = runtime
        .build(&plan, CancellationToken::new())
        .map_err(build_error)?;
    packages.revalidate().map_err(build_error)?;
    receipt::write(
        &receipt,
        receipt_path,
        project_path,
        &root,
        &authored,
        &plan,
        &roots,
    )?;
    if receipt.outcome != ProjectBuildOutcome::Succeeded {
        return Err(CliError::new(
            "PROJECT_BUILD_FAILED",
            format!("project build completed with outcome {:?}", receipt.outcome),
        ));
    }
    match mode {
        ExecutionMode::Build => Ok(()),
        ExecutionMode::Evidence => evidence_result::inspect(&receipt, &store).map(|_| ()),
        ExecutionMode::Test => gate_evidence(evidence_result::inspect(&receipt, &store)?),
    }
}

pub(super) fn validate_package_authorities(
    root: &Path,
    paths: &veac_project::ProjectPaths,
    packages: &veac_build::ProjectPackageSet,
) -> CliResult {
    paths::validate_package_authorities(root, paths, packages)
}

fn gate_evidence(summary: evidence_result::EvidenceSummary) -> CliResult {
    if summary.is_pass() {
        Ok(())
    } else {
        Err(CliError::new(
            "PROJECT_TEST_FAILED",
            format!(
                "evidence assertions did not pass: {} passed, {} failed, {} errors",
                summary.passed, summary.failed, summary.errors
            ),
        ))
    }
}

pub(super) fn build_error(error: veac_build::BuildError) -> CliError {
    let code = match error.kind() {
        veac_build::BuildErrorKind::InvalidContract => "PROJECT_BUILD_CONTRACT",
        veac_build::BuildErrorKind::ResourceLimit => "PROJECT_BUILD_LIMIT",
        veac_build::BuildErrorKind::Cache => "PROJECT_BUILD_CACHE",
        veac_build::BuildErrorKind::Cancelled => "PROJECT_BUILD_CANCELLED",
        veac_build::BuildErrorKind::Internal => "PROJECT_BUILD_INTERNAL",
    };
    if error.kind() == veac_build::BuildErrorKind::ResourceLimit {
        CliError::resource_limit(code, error.to_string())
    } else {
        CliError::new(code, error.to_string())
    }
}

#[cfg(test)]
#[path = "execution/tests.rs"]
mod tests;
