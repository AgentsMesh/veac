use std::collections::VecDeque;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::path::Path;

use rustix::fs::{open, Mode};

use super::{create_directory, open_directory, path_parts, BoundChain, DIRECTORY_FLAGS};
use crate::cache::io::common::{corrupt_io, io_error};
use crate::ArtifactResult;

struct Part {
    name: OsString,
    create: bool,
    allow_symlink: bool,
}

impl BoundChain {
    pub(in crate::cache::io) fn root(path: &Path, create: bool) -> ArtifactResult<Option<Self>> {
        let (base, names, normalized) = path_parts(path)?;
        let anchor = open(base, DIRECTORY_FLAGS, Mode::empty())
            .map(File::from)
            .map_err(|error| io_error("open cache path anchor", error))?;
        let count = names.len();
        let mut parts = names
            .into_iter()
            .enumerate()
            .map(|(index, name)| Part {
                name,
                create: create && index + 1 == count,
                allow_symlink: index + 1 < count,
            })
            .collect::<VecDeque<_>>();
        let mut chain = Self {
            anchor,
            nodes: Vec::new(),
            symlinks: Vec::new(),
            path: normalized,
        };
        let mut followed = 0_usize;
        while let Some(part) = parts.pop_front() {
            match open_directory(chain.current(), &part.name) {
                Ok(file) => chain.append(&part.name, file),
                Err(rustix::io::Errno::NOENT) if part.create => {
                    let file = create_directory(chain.current(), &part.name)?;
                    chain.append(&part.name, file);
                }
                Err(rustix::io::Errno::NOENT) if !create => return Ok(None),
                Err(rustix::io::Errno::NOENT) => {
                    return Err(io_error(
                        "open cache root ancestor",
                        rustix::io::Errno::NOENT,
                    ))
                }
                Err(error) if part.allow_symlink && is_link_error(error) => {
                    let Some((guard, target)) =
                        super::symlink::resolve(chain.current(), &part.name)?
                    else {
                        return Err(component_error(&part.name, error));
                    };
                    followed += 1;
                    if followed > 40 {
                        return crate::cache::io::common::unsafe_path(
                            "cache root has too many symbolic-link ancestors",
                        );
                    }
                    chain.symlinks.push(guard);
                    prepend(&mut parts, target);
                }
                Err(error) => return Err(component_error(&part.name, error)),
            }
        }
        Ok(Some(chain))
    }
}

fn prepend(parts: &mut VecDeque<Part>, target: Vec<OsString>) {
    for name in target.into_iter().rev() {
        parts.push_front(Part {
            name,
            create: false,
            allow_symlink: true,
        });
    }
}

fn is_link_error(error: rustix::io::Errno) -> bool {
    matches!(error, rustix::io::Errno::LOOP | rustix::io::Errno::NOTDIR)
}

fn component_error(name: &OsStr, error: rustix::io::Errno) -> crate::ArtifactError {
    corrupt_io(&format!("open cache directory component {name:?}"), error)
}
