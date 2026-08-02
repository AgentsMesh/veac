mod identity;
mod snapshot;

use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use rustix::fd::OwnedFd;
use rustix::fs::{open, openat, Mode, OFlags};

use super::path::{confined_request, normalize};
use super::{LoadedSource, SourceLoader};
use identity::{FileIdentity, IdentityRegistry};
use snapshot::read_regular;

#[derive(Debug, Clone)]
pub struct FileSystemLoader {
    root: PathBuf,
    directory: Arc<OwnedFd>,
    identities: IdentityRegistry,
}

impl FileSystemLoader {
    pub fn for_entry(entry: &Path) -> Result<(Self, LoadedSource), String> {
        let parent = entry
            .parent()
            .filter(|value| !value.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let root = fs::canonicalize(parent)
            .map_err(|error| format!("cannot resolve source root {}: {error}", parent.display()))?;
        let directory = open_directory(&root, &root)?;
        let name = entry
            .file_name()
            .ok_or_else(|| format!("entry {} is not a regular file", entry.display()))?;
        let id = normalize(Path::new(name))?;
        let opened = open_source(&directory, Path::new(&id), entry)?;
        let identities = IdentityRegistry::default();
        identities.register(&id, opened.identity)?;
        let loader = Self {
            root,
            directory: Arc::new(directory),
            identities,
        };
        Ok((
            loader,
            LoadedSource {
                id,
                source: opened.source,
            },
        ))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl SourceLoader for FileSystemLoader {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String> {
        let requested = confined_request(requested)?;
        let parent = Path::new(importer)
            .parent()
            .unwrap_or_else(|| Path::new(""));
        let id = normalize(&parent.join(requested))?;
        let opened = open_source(&self.directory, Path::new(&id), &self.root.join(&id))?;
        self.identities.register(&id, opened.identity)?;
        Ok(LoadedSource {
            id,
            source: opened.source,
        })
    }
}

struct OpenedSource {
    source: String,
    identity: FileIdentity,
}

fn open_source(root: &OwnedFd, relative: &Path, label: &Path) -> Result<OpenedSource, String> {
    let mut parts = relative.components().peekable();
    let mut directory = rustix::io::dup(root)
        .map_err(|error| format!("cannot duplicate source root descriptor: {error}"))?;
    while let Some(component) = parts.next() {
        let Component::Normal(name) = component else {
            return Err("module import escapes the source root".to_owned());
        };
        if parts.peek().is_some() {
            directory = open_directory_at(&directory, Path::new(name), label)?;
        } else {
            let file = openat(
                &directory,
                Path::new(name),
                OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|error| unsafe_path(label, error))?;
            return read_regular(file, &directory, Path::new(name), label);
        }
    }
    Err(format!("source {} has an empty path", label.display()))
}

fn open_directory(path: &Path, label: &Path) -> Result<OwnedFd, String> {
    open(
        path,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| unsafe_path(label, error))
}

fn open_directory_at(root: &OwnedFd, path: &Path, label: &Path) -> Result<OwnedFd, String> {
    openat(
        root,
        path,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| unsafe_path(label, error))
}

fn unsafe_path(label: &Path, error: rustix::io::Errno) -> String {
    format!(
        "source {} escapes the source root, uses a symlink, or is unavailable: {error}",
        label.display()
    )
}
