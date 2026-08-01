use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use veac_codegen::emitter::BackendAction;

use super::super::model::RuntimeBundle;
use super::invalid;
use crate::RuntimeError;

mod aliases;
pub(super) mod declaration;
mod existing;

pub(super) fn validate(
    bundle: &RuntimeBundle,
    protected: &BTreeSet<PathBuf>,
) -> Result<PathBuf, RuntimeError> {
    validate_command_inputs(bundle, protected)?;
    let declarations = declaration::collect(bundle)?;
    for (index, declaration) in declarations.iter().enumerate() {
        if declarations[..index]
            .iter()
            .any(|existing| declaration::overlaps(existing, declaration))
        {
            return invalid("backend output paths or image patterns overlap");
        }
        if protected
            .iter()
            .any(|path| declaration::consumes(declaration, path))
        {
            return invalid("backend output aliases a protected input or resource");
        }
    }
    let parents = declarations
        .iter()
        .map(declaration::parent)
        .collect::<BTreeSet<_>>();
    if parents.len() != 1 {
        return invalid("one backend bundle must publish into one output directory");
    }
    Ok(parents.into_iter().next().unwrap())
}

pub(super) fn validate_filesystem(
    bundle: &RuntimeBundle,
    protected: &BTreeSet<PathBuf>,
) -> Result<(), RuntimeError> {
    aliases::validate(&declaration::collect(bundle)?, protected)
}

fn validate_command_inputs(
    bundle: &RuntimeBundle,
    protected: &BTreeSet<PathBuf>,
) -> Result<(), RuntimeError> {
    for command in bundle.tasks.iter().filter_map(|task| match &task.action {
        BackendAction::Ffmpeg(command) => Some(command),
        BackendAction::WriteFile { .. } => None,
    }) {
        for input in &command.inputs {
            let canonical = fs::canonicalize(&input.path).map_err(path_error)?;
            if !protected.contains(&canonical) {
                return invalid("every FFmpeg input must be a protected bundle resource");
            }
        }
    }
    Ok(())
}

pub(super) fn utf8(path: &Path) -> Result<&str, RuntimeError> {
    path.to_str()
        .ok_or_else(|| RuntimeError::new("backend paths must be valid UTF-8"))
}

pub(in crate::executor::contract) fn path_error(error: std::io::Error) -> RuntimeError {
    RuntimeError::new(format!("cannot validate protected backend path: {error}"))
}
