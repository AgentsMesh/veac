use std::path::Path;

use crate::error::CliResult;

pub(crate) fn run(
    file: &Path,
    inputs: Option<&Path>,
    inline_inputs: &[String],
    revision: u64,
) -> CliResult {
    let project = crate::frontend::check(file, inputs, inline_inputs, revision)?;
    println!(
        "Executable source is valid: {} (project {}, revision {})",
        file.display(),
        project.project.id,
        project.project.revision
    );
    Ok(())
}
