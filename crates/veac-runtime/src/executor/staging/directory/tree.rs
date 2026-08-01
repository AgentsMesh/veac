use rustix::fs::fstat;

use super::entry::from_stat;
use super::{Directory, EntryIdentity, EntryState};
use crate::RuntimeError;

impl Directory {
    pub(in crate::executor) fn sync_tree_bound(
        &self,
        name: &str,
        expected: EntryIdentity,
        maximum: usize,
        mut guard: impl FnMut() -> bool,
    ) -> Result<(), RuntimeError> {
        let mut visited = 0;
        sync_node(self, name, expected, maximum, &mut visited, &mut guard)
    }

    pub(in crate::executor) fn remove_tree_bound(
        &self,
        name: &str,
        expected: EntryIdentity,
        maximum: usize,
        mut guard: impl FnMut() -> bool,
    ) -> Result<(), RuntimeError> {
        let mut visited = 0;
        remove_node(self, name, expected, maximum, &mut visited, &mut guard)
    }

    fn child_bound(&self, name: &str, expected: EntryIdentity) -> Result<Directory, RuntimeError> {
        self.require(name, expected)?;
        let child = self.child(name)?;
        let actual = fstat(&child.descriptor)
            .map_err(|error| super::failure(&child.path, "inspect opened directory", error))?;
        if from_stat(actual)? != expected {
            return Err(RuntimeError::new(
                "opened package directory changed identity",
            ));
        }
        Ok(child)
    }
}

fn sync_node(
    parent: &Directory,
    name: &str,
    expected: EntryIdentity,
    maximum: usize,
    visited: &mut usize,
    guard: &mut impl FnMut() -> bool,
) -> Result<(), RuntimeError> {
    active(guard)?;
    count(visited, maximum)?;
    match expected {
        EntryIdentity::Regular { .. } => parent.sync_bound(name, expected),
        EntryIdentity::Directory { .. } => {
            let child = parent.child_bound(name, expected)?;
            for entry in child.entries(maximum.saturating_sub(*visited))? {
                active(guard)?;
                let entry = utf8(&entry)?;
                match child.state(entry)? {
                    EntryState::Regular(identity) | EntryState::Directory(identity) => {
                        sync_node(&child, entry, identity, maximum, visited, guard)?
                    }
                    EntryState::Missing => return changed(),
                }
            }
            child.sync()?;
            parent.require(name, expected)
        }
    }
}

fn remove_node(
    parent: &Directory,
    name: &str,
    expected: EntryIdentity,
    maximum: usize,
    visited: &mut usize,
    guard: &mut impl FnMut() -> bool,
) -> Result<(), RuntimeError> {
    active(guard)?;
    count(visited, maximum)?;
    match expected {
        EntryIdentity::Regular { .. } => parent.remove_bound(name, expected),
        EntryIdentity::Directory { .. } => {
            let child = parent.child_bound(name, expected)?;
            for entry in child.entries(maximum.saturating_sub(*visited))? {
                let entry = utf8(&entry)?;
                match child.state(entry)? {
                    EntryState::Regular(identity) | EntryState::Directory(identity) => {
                        remove_node(&child, entry, identity, maximum, visited, guard)?
                    }
                    EntryState::Missing => return changed(),
                }
            }
            parent.remove_child(name, &child)
        }
    }
}

fn count(visited: &mut usize, maximum: usize) -> Result<(), RuntimeError> {
    *visited = visited.saturating_add(1);
    if *visited > maximum {
        Err(RuntimeError::resource_limit(
            "package tree exceeds its operation budget",
        ))
    } else {
        Ok(())
    }
}

fn active(guard: &mut impl FnMut() -> bool) -> Result<(), RuntimeError> {
    if guard() {
        Ok(())
    } else {
        Err(RuntimeError::resource_limit(
            "package tree operation exceeded its deadline",
        ))
    }
}

fn utf8(value: &std::ffi::OsStr) -> Result<&str, RuntimeError> {
    value
        .to_str()
        .ok_or_else(|| RuntimeError::new("package tree entry name must be valid UTF-8"))
}

fn changed<T>() -> Result<T, RuntimeError> {
    Err(RuntimeError::new(
        "package tree changed during descriptor-relative operation",
    ))
}
