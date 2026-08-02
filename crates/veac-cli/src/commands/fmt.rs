use std::path::Path;

use crate::error::{CliError, CliResult};

pub(crate) fn run(file: &Path, check: bool, stdout: bool) -> CliResult {
    let location = crate::fs::SourceLocation::resolve(file)?;
    let (source, formatted) = crate::frontend::format(location.path())?;
    if check {
        if source == formatted {
            println!("Source is canonically formatted: {}", file.display());
            return Ok(());
        }
        return Err(CliError::new(
            "FORMAT_REQUIRED",
            format!("{} is not canonically formatted", file.display()),
        ));
    }
    if stdout {
        return crate::fs::write_stdout(&formatted);
    }
    if source != formatted {
        let source_lock = crate::fs::SourceGraphLock::acquire(location.root())?;
        source_lock.commit_module(location.root(), location.module(), &source, &formatted)?;
    }
    println!("Formatted: {}", file.display());
    Ok(())
}
