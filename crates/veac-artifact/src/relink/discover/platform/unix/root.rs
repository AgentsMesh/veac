use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Component, Path, PathBuf};

use rustix::fs::{fstat, open, openat, Mode};

use super::state::{io_error, open_error, require_same};
use super::{BoundDirectory, DIRECTORY_FLAGS};
use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult};

use super::super::super::DiscoveryBudget;

pub(crate) struct BoundRoot {
    directory: BoundDirectory,
}

impl BoundRoot {
    pub(crate) fn directory(&self) -> &BoundDirectory {
        &self.directory
    }

    pub(crate) fn path(&self) -> &Path {
        self.directory.path()
    }

    pub(crate) fn verify(&self, budget: &mut DiscoveryBudget) -> ArtifactResult<()> {
        let sealed = self.directory.seal(budget)?;
        let current = bind_absolute(self.path(), budget)?;
        let state = fstat(&current).map_err(io_error)?;
        budget.check_deadline()?;
        require_same(&sealed.state, &state)
    }
}

pub(crate) fn bind_roots(
    roots: &[PathBuf],
    budget: &mut DiscoveryBudget,
) -> ArtifactResult<Vec<BoundRoot>> {
    let mut bound = BTreeMap::new();
    for root in roots {
        budget.check_deadline()?;
        let initial = open(root, DIRECTORY_FLAGS, Mode::empty())
            .map(File::from)
            .map_err(open_error)?;
        budget.check_deadline()?;
        let canonical = std::fs::canonicalize(root)?;
        budget.check_deadline()?;
        let file = bind_absolute(&canonical, budget)?;
        let initial_state = fstat(&initial).map_err(io_error)?;
        budget.check_deadline()?;
        let bound_state = fstat(&file).map_err(io_error)?;
        budget.check_deadline()?;
        require_same(&initial_state, &bound_state)?;
        let directory = BoundDirectory::new(file, canonical.clone())?;
        bound.entry(canonical).or_insert(BoundRoot { directory });
    }
    Ok(bound.into_values().collect())
}

fn bind_absolute(path: &Path, budget: &DiscoveryBudget) -> ArtifactResult<File> {
    if !path.is_absolute() {
        return unsafe_path("canonical relink root must be absolute");
    }
    budget.check_deadline()?;
    let mut current = open("/", DIRECTORY_FLAGS, Mode::empty())
        .map(File::from)
        .map_err(open_error)?;
    for component in path.components() {
        let Component::Normal(name) = component else {
            if matches!(component, Component::RootDir) {
                continue;
            }
            return unsafe_path("canonical relink root contains an unsafe path component");
        };
        budget.check_deadline()?;
        current = openat(&current, name, DIRECTORY_FLAGS, Mode::empty())
            .map(File::from)
            .map_err(open_error)?;
    }
    budget.check_deadline()?;
    Ok(current)
}

fn unsafe_path<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(ArtifactErrorKind::UnsafePath, message))
}

#[cfg(test)]
#[path = "root/tests.rs"]
mod tests;
