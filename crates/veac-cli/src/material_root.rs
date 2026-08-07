use std::path::{Path, PathBuf};

use veac_ir::{MaterialSource, ProjectEnvelope};

use crate::error::{CliError, CliResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MaterialRootOrigin {
    SourceDefault,
    ProjectDefault,
    Explicit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MaterialRoot {
    path: PathBuf,
    origin: MaterialRootOrigin,
}

impl MaterialRoot {
    pub(crate) fn for_build(source_root: &Path, requested: Option<&Path>) -> CliResult<Self> {
        Self::resolve(source_root, requested, MaterialRootOrigin::SourceDefault)
    }

    pub(crate) fn for_project(project: &Path, requested: Option<&Path>) -> CliResult<Self> {
        let default = project
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        Self::resolve(default, requested, MaterialRootOrigin::ProjectDefault)
    }

    fn resolve(
        default: &Path,
        requested: Option<&Path>,
        default_origin: MaterialRootOrigin,
    ) -> CliResult<Self> {
        let path = crate::fs::canonical_directory(requested.unwrap_or(default), "material root")?;
        let origin = requested
            .map(|_| MaterialRootOrigin::Explicit)
            .unwrap_or(default_origin);
        Ok(Self { path, origin })
    }

    pub(crate) fn resolve_file(&self, uri: &str) -> CliResult<PathBuf> {
        let file = crate::fs::canonical_file(&self.path().join(uri), "material")?;
        if !file.starts_with(self.path()) {
            return Err(CliError::new(
                "MATERIAL_OUTSIDE_ROOT",
                format!(
                    "material {uri} resolves outside material root {}",
                    self.path().display()
                ),
            ));
        }
        Ok(file)
    }

    pub(crate) fn local_paths(&self, envelope: &ProjectEnvelope) -> Vec<PathBuf> {
        envelope
            .project
            .materials
            .iter()
            .filter_map(|material| match &material.source {
                MaterialSource::File { uri } => Some(self.path().join(uri)),
                MaterialSource::Remote { .. } => None,
            })
            .collect()
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn is_explicit(&self) -> bool {
        self.origin == MaterialRootOrigin::Explicit
    }
}

pub(crate) fn require_detached_output_contract(
    envelope: &ProjectEnvelope,
    source_root: &Path,
    material_root: &MaterialRoot,
    output: &Path,
) -> CliResult {
    let has_local = envelope
        .project
        .materials
        .iter()
        .any(|material| matches!(&material.source, MaterialSource::File { .. }));
    let output_root = output.parent().unwrap_or(Path::new("."));
    if has_local && output_root != source_root && !material_root.is_explicit() {
        return Err(CliError::new(
            "OUTPUT_MATERIAL_BASE_MISMATCH",
            format!(
                "canonical IR with local materials requires source-root output or --material-root {}",
                source_root.display()
            ),
        ));
    }
    Ok(())
}
