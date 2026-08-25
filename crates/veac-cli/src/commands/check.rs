use std::path::Path;
use std::path::PathBuf;

use crate::error::CliResult;

pub(crate) fn run(
    file: &Path,
    inputs: Option<&Path>,
    inline_inputs: &[String],
    package_roots: &[PathBuf],
    revision: u64,
) -> CliResult {
    let project = crate::frontend::check(file, inputs, inline_inputs, package_roots, revision)?;
    println!(
        "Executable source is valid: {} (project {}, revision {})",
        file.display(),
        project.project.id,
        project.project.revision
    );
    Ok(())
}
