use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
};

use sha2::{Digest, Sha256};
use veac_artifact::{ContentDigest, DigestAlgorithm};
use veac_project::ProjectPath;

use crate::{BuildError, BuildResult, ProjectFileSnapshot};

mod evidence_graph;
mod package;
mod source_graph;

pub use package::ProjectPackageSet;

pub(super) struct ProjectRoots {
    source: PathBuf,
    material: PathBuf,
    packages: ProjectPackageSet,
}

impl ProjectRoots {
    pub fn new(
        source: impl Into<PathBuf>,
        material: impl Into<PathBuf>,
        packages: ProjectPackageSet,
    ) -> BuildResult<Self> {
        Ok(Self {
            source: checked_root(source.into(), "source base")?,
            material: checked_root(material.into(), "material root")?,
            packages,
        })
    }

    pub fn material(&self, path: &ProjectPath) -> BuildResult<ProjectFileSnapshot> {
        snapshot(&self.material, path)
    }

    pub fn package_revision(&self) -> Vec<crate::ProjectPackageMountRevision> {
        self.packages.revision().to_vec()
    }

    pub fn revalidate_packages(&self) -> BuildResult<()> {
        self.packages.revalidate()
    }

    pub fn source_graph(
        &self,
        path: &ProjectPath,
    ) -> BuildResult<(ProjectFileSnapshot, crate::ProjectSourceGraphRevision)> {
        source_graph::capture(&self.source, path, &self.packages)
    }

    pub fn evidence_graph(
        &self,
        path: &ProjectPath,
    ) -> BuildResult<(ProjectFileSnapshot, crate::ProjectSourceGraphRevision)> {
        evidence_graph::capture(&self.source, path, &self.packages)
    }
}

fn checked_root(path: PathBuf, label: &str) -> BuildResult<PathBuf> {
    let metadata = std::fs::symlink_metadata(&path)
        .map_err(|error| io_error(format!("cannot inspect {label}"), error))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(BuildError::invalid(format!(
            "project {label} must be a non-symlink directory"
        )));
    }
    std::fs::canonicalize(path).map_err(|error| io_error(format!("cannot open {label}"), error))
}

fn snapshot(root: &Path, project_path: &ProjectPath) -> BuildResult<ProjectFileSnapshot> {
    let path = checked_file(root, project_path)?;
    let (content, size_bytes) = fingerprint(&path)?;
    Ok(ProjectFileSnapshot {
        path: project_path.as_str().to_owned(),
        content,
        size_bytes,
    })
}

fn checked_file(root: &Path, project_path: &ProjectPath) -> BuildResult<PathBuf> {
    let relative = Path::new(project_path.as_str());
    if relative.is_absolute()
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(BuildError::invalid(
            "project source path is not canonical and relative",
        ));
    }
    reject_symlink_components(root, relative)?;
    let path = root.join(relative);
    let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
        io_error(
            format!("cannot inspect project source {}", project_path.as_str()),
            error,
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(BuildError::invalid(format!(
            "project source {} must be a regular non-symlink file",
            project_path.as_str()
        )));
    }
    Ok(path)
}

fn reject_symlink_components(root: &Path, relative: &Path) -> BuildResult<()> {
    let mut current = root.to_owned();
    for component in relative.components() {
        current.push(component.as_os_str());
        if std::fs::symlink_metadata(&current)
            .map_err(|error| io_error("cannot inspect project source component", error))?
            .file_type()
            .is_symlink()
        {
            return Err(BuildError::invalid(
                "project source path contains a symlink",
            ));
        }
    }
    Ok(())
}

fn fingerprint(path: &Path) -> BuildResult<(ContentDigest, u64)> {
    let mut file =
        File::open(path).map_err(|error| io_error("cannot open project source", error))?;
    let mut hash = Sha256::new();
    let mut size = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| io_error("cannot read project source", error))?;
        if read == 0 {
            break;
        }
        size = size
            .checked_add(read as u64)
            .ok_or_else(|| BuildError::resource_limit("project source size overflow"))?;
        hash.update(&buffer[..read]);
    }
    Ok((
        ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: format!("{:x}", hash.finalize()),
        },
        size,
    ))
}

fn graph_root_source<'a>(
    sources: &'a BTreeMap<String, String>,
    root: &str,
    label: &str,
) -> BuildResult<&'a str> {
    match sources.get(root) {
        Some(source) => Ok(source),
        None => Err(BuildError::invalid(format!(
            "{label} source graph omitted its root module"
        ))),
    }
}

fn graph_module_count(length: usize, label: &str) -> BuildResult<u32> {
    match u32::try_from(length) {
        Ok(count) => Ok(count),
        Err(_) => Err(BuildError::resource_limit(format!(
            "{label} module count overflow"
        ))),
    }
}

fn io_error(context: impl Into<String>, error: std::io::Error) -> BuildError {
    BuildError::new(
        crate::BuildErrorKind::InvalidContract,
        format!("{}: {error}", context.into()),
    )
}

#[cfg(test)]
#[path = "snapshot/coverage_tests.rs"]
mod coverage_tests;

#[cfg(test)]
#[path = "snapshot/tests.rs"]
mod tests;
