use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod render;
mod token;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendFilterEscape {
    FilterValue,
    Quoted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendInternalAccess {
    Produce,
    Consume,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendFilterBinding {
    File {
        token: String,
        path: PathBuf,
        escape: BackendFilterEscape,
    },
    Directory {
        token: String,
        directory: PathBuf,
        files: Vec<PathBuf>,
        escape: BackendFilterEscape,
    },
    InternalFile {
        token: String,
        path: PathBuf,
        access: BackendInternalAccess,
        escape: BackendFilterEscape,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendFilterContract {
    template: String,
    bindings: Vec<BackendFilterBinding>,
}

impl BackendFilterBinding {
    pub fn file(token: String, path: PathBuf, escape: BackendFilterEscape) -> Self {
        Self::File {
            token,
            path,
            escape,
        }
    }

    pub fn directory(
        token: String,
        directory: PathBuf,
        files: Vec<PathBuf>,
        escape: BackendFilterEscape,
    ) -> Self {
        Self::Directory {
            token,
            directory,
            files,
            escape,
        }
    }

    pub fn internal_file(
        token: String,
        path: PathBuf,
        access: BackendInternalAccess,
        escape: BackendFilterEscape,
    ) -> Self {
        Self::InternalFile {
            token,
            path,
            access,
            escape,
        }
    }

    pub fn files(&self) -> &[PathBuf] {
        match self {
            Self::File { path, .. } => std::slice::from_ref(path),
            Self::Directory { files, .. } => files,
            Self::InternalFile { path, .. } => std::slice::from_ref(path),
        }
    }

    pub fn token(&self) -> &str {
        match self {
            Self::File { token, .. }
            | Self::Directory { token, .. }
            | Self::InternalFile { token, .. } => token,
        }
    }

    fn escape(&self) -> BackendFilterEscape {
        match self {
            Self::File { escape, .. }
            | Self::Directory { escape, .. }
            | Self::InternalFile { escape, .. } => *escape,
        }
    }

    fn original(&self) -> &Path {
        match self {
            Self::File { path, .. } => path,
            Self::Directory { directory, .. } => directory,
            Self::InternalFile { path, .. } => path,
        }
    }
}

impl BackendFilterContract {
    pub fn new(template: String, bindings: Vec<BackendFilterBinding>) -> Result<Self, String> {
        let value = Self { template, bindings };
        value.validate_tokens()?;
        Ok(value)
    }

    pub fn bindings(&self) -> &[BackendFilterBinding] {
        &self.bindings
    }

    pub fn template(&self) -> &str {
        &self.template
    }

    pub fn render_original(&self) -> Result<String, String> {
        render::original(self)
    }

    pub fn render_bound(
        &self,
        files: &BTreeMap<PathBuf, PathBuf>,
        directories: &BTreeMap<Vec<PathBuf>, PathBuf>,
    ) -> Result<String, String> {
        render::bound(self, files, directories)
    }

    pub fn render_internal(&self, graph: &str, root: &Path) -> Result<String, String> {
        render::internal(self, graph, root)
    }

    fn validate_tokens(&self) -> Result<(), String> {
        token::validate(&self.template, &self.bindings)
    }
}

#[cfg(test)]
mod internal_tests;
#[cfg(test)]
mod tests;
