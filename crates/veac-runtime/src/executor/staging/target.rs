use std::path::{Path, PathBuf};

use veac_codegen::emitter::BackendOutput;

use super::StagedOutput;
use crate::executor::output;
use crate::RuntimeError;

pub(in crate::executor) fn common_parent_from_outputs(
    outputs: &[StagedOutput],
) -> Result<&Path, RuntimeError> {
    outputs
        .first()
        .and_then(|output| output.target().parent())
        .ok_or_else(|| RuntimeError::new("staged task has no output parent"))
}

pub(in crate::executor) fn common_parent(output: &BackendOutput) -> Result<&Path, RuntimeError> {
    let paths: Vec<&Path> = match output {
        BackendOutput::File(path) => vec![path],
        BackendOutput::Files { paths } => paths.iter().map(PathBuf::as_path).collect(),
        BackendOutput::ImageSequence { pattern } => vec![pattern],
        BackendOutput::Package { root, .. } => vec![root],
    };
    let parent = output::ensure_safe_parent(paths[0])?;
    if paths
        .iter()
        .all(|path| output::ensure_safe_parent(path).is_ok_and(|value| value == parent))
    {
        Ok(parent)
    } else {
        Err(RuntimeError::new(
            "one backend task may not span output directories",
        ))
    }
}
