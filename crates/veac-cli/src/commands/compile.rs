use std::path::Path;

use crate::error::{CliError, CliResult};

pub(crate) fn run(source: &Path, emit_ir: Option<&Path>, revision: u64) -> CliResult {
    let envelope = crate::frontend::compile(source, revision)?;
    let mut json = match veac_ir::canonical_json(&envelope) {
        Ok(json) => json,
        Err(error) => return Err(CliError::new("CANONICAL_ENCODE", error.to_string())),
    };
    json.push('\n');
    let Some(destination) = emit_ir else {
        return crate::fs::write_stdout(&json);
    };
    if destination == Path::new("-") {
        return crate::fs::write_stdout(&json);
    }
    let source = crate::fs::canonical_file(source, "source")?;
    let mut protected = vec![source.clone()];
    protected.extend(crate::canonical::local_material_paths(&envelope, &source));
    let destination = crate::output::guarded_write_many(
        destination,
        protected.iter().map(std::path::PathBuf::as_path),
    )?;
    crate::fs::atomic_write(&destination, &json)?;
    println!("Canonical IR written: {}", destination.display());
    Ok(())
}
