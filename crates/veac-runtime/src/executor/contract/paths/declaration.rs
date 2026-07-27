mod image;
mod matching;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use veac_codegen::emitter::{BackendOutput, BackendPhase, BackendTask};

use super::{existing, utf8};
use crate::executor::model::RuntimeBundle;
use crate::executor::output;
use crate::RuntimeError;

pub(super) use matching::{consumes, overlaps};

pub(in crate::executor::contract) enum Declaration {
    Static(PathBuf),
    Pattern {
        parent: PathBuf,
        prefix: Vec<u8>,
        suffix: Vec<u8>,
        minimum_width: Option<usize>,
    },
    Passlog {
        parent: PathBuf,
        prefix: Vec<u8>,
    },
}

pub(in crate::executor::contract) fn collect(
    bundle: &RuntimeBundle,
) -> Result<Vec<Declaration>, RuntimeError> {
    let mut values = Vec::new();
    for task in &bundle.tasks {
        if task.phase == BackendPhase::FirstPass {
            values.push(passlog(task)?);
            continue;
        }
        match &task.output {
            BackendOutput::File(path) => values.push(static_path(path)?),
            BackendOutput::Files { paths } => {
                for path in paths {
                    values.push(static_path(path)?);
                }
            }
            BackendOutput::ImageSequence { pattern } => values.push(image::from_path(pattern)?),
        }
    }
    Ok(values)
}

pub(in crate::executor::contract) fn overlaps_root(declaration: &Declaration, root: &Path) -> bool {
    match declaration {
        Declaration::Static(path) => path.starts_with(root) || root.starts_with(path),
        Declaration::Pattern { parent, .. } | Declaration::Passlog { parent, .. } => {
            parent.starts_with(root)
                || root
                    .ancestors()
                    .any(|candidate| consumes(declaration, candidate))
        }
    }
}

pub(in crate::executor::contract) fn parent(declaration: &Declaration) -> PathBuf {
    match declaration {
        Declaration::Static(path) => path.parent().unwrap().to_path_buf(),
        Declaration::Pattern { parent, .. } | Declaration::Passlog { parent, .. } => parent.clone(),
    }
}

pub(in crate::executor::contract) fn parents(declarations: &[Declaration]) -> BTreeSet<PathBuf> {
    declarations.iter().map(parent).collect()
}

fn static_path(path: &Path) -> Result<Declaration, RuntimeError> {
    existing::validate_static(path)?;
    Ok(Declaration::Static(normalized(path)?))
}

fn passlog(task: &BackendTask) -> Result<Declaration, RuntimeError> {
    let normalized = normalized(&output::passlog_prefix(task)?)?;
    output::enumerate_passlogs(&normalized)?;
    Ok(Declaration::Passlog {
        parent: normalized.parent().unwrap().to_path_buf(),
        prefix: normalized.file_name().unwrap().as_encoded_bytes().to_vec(),
    })
}

fn normalized(path: &Path) -> Result<PathBuf, RuntimeError> {
    utf8(path)?;
    let parent = output::ensure_safe_parent(path)?;
    let parent = fs::canonicalize(parent).map_err(super::path_error)?;
    Ok(parent.join(path.file_name().unwrap()))
}
