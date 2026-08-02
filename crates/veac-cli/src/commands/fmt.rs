use std::path::Path;

use crate::error::{CliError, CliResult};

pub(crate) fn run(file: &Path, check: bool, stdout: bool) -> CliResult {
    let (source, formatted) = crate::frontend::format(file)?;
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
        let canonical = crate::fs::canonical_file(file, "source")?;
        crate::fs::atomic_write(&canonical, &formatted)?;
    }
    println!("Formatted: {}", file.display());
    Ok(())
}
