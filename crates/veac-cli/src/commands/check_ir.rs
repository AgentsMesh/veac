use std::path::Path;

use crate::error::CliResult;

pub(crate) fn run(file: &Path) -> CliResult {
    let envelope = crate::canonical::load(file)?;
    println!(
        "Canonical IR is valid: {} (project {}, schema version {})",
        file.display(),
        envelope.project.id,
        envelope.schema_version
    );
    Ok(())
}
