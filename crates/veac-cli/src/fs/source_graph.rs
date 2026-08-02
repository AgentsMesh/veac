use std::path::Path;

use crate::error::{CliError, CliResult};
use veac_lang::program::{FileSystemLoader, SourceLoader};
use veac_lang::source_edit::{source_graph_revision, SourceModule, SourceRevision};

pub(crate) fn ensure_source_graph_unchanged(
    root: &Path,
    entry_module: &str,
    expected_modules: &[String],
    expected_revision: &SourceRevision,
) -> CliResult {
    let (loader, entry) = FileSystemLoader::for_entry(&root.join(entry_module))
        .map_err(|_| changed(&root.join(entry_module)))?;
    if !expected_modules.iter().any(|module| module == entry_module) {
        return Err(changed(&root.join(entry_module)));
    }
    let mut loaded = Vec::with_capacity(expected_modules.len());
    for module in expected_modules {
        veac_lang::source_edit::validate_module_path(module)
            .map_err(|_| changed(&root.join(module)))?;
        let path = root.join(module);
        let actual = if module == entry_module {
            entry.clone()
        } else {
            loader
                .load(entry_module, module)
                .map_err(|_| changed(&path))?
        };
        if actual.id != *module {
            return Err(changed(&path));
        }
        loaded.push(actual);
    }
    let modules = loaded
        .iter()
        .map(|source| SourceModule::utf8(&source.id, &source.source))
        .collect::<Vec<_>>();
    let actual = source_graph_revision(&modules).map_err(|_| changed(&root.join(entry_module)))?;
    if actual != *expected_revision {
        return Err(changed(&root.join(entry_module)));
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
