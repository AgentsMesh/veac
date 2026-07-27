use std::path::Path;

use crate::error::CliResult;

pub(crate) fn run(file: &Path, revision: u64) -> CliResult {
    let project = crate::frontend::compile(file, revision)?;
    println!(
        "Source is valid: {} (project {}, revision {})",
        file.display(),
        project.project.id,
        project.project.revision
    );
    Ok(())
}
