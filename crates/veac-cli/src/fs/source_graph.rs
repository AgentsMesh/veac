use std::path::Path;

use crate::error::{CliError, CliResult};
use veac_lang::program::{FileSystemLoader, SourceLoader};

pub(crate) fn ensure_source_modules_unchanged(
    root: &Path,
    entry_module: &str,
    expected: &[(&str, &str)],
) -> CliResult {
    let (loader, entry) = FileSystemLoader::for_entry(&root.join(entry_module))
        .map_err(|_| changed(&root.join(entry_module)))?;
    for (module, source) in expected {
        veac_lang::source_edit::validate_module_path(module)
            .map_err(|_| changed(&root.join(module)))?;
        let actual = if *module == entry_module {
            entry.clone()
        } else {
            loader
                .load(entry_module, module)
                .map_err(|_| changed(&root.join(module)))?
        };
        if actual.id != *module || actual.source != *source {
            return Err(changed(&root.join(module)));
        }
    }
    Ok(())
}

fn changed(path: &Path) -> CliError {
    CliError::new(
        "SOURCE_CHANGED",
        format!(
            "source graph module {} changed after the edit was validated",
            path.display()
        ),
    )
}

#[cfg(test)]
#[path = "source_graph/tests.rs"]
mod tests;
