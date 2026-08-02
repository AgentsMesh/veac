use std::path::Path;

use super::path;
use super::stage::Staged;
use super::SourceGraphLock;
use crate::error::{CliError, CliResult};

impl SourceGraphLock {
    pub(crate) fn commit_module(
        &self,
        root: &Path,
        module: &str,
        expected: &str,
        replacement: &str,
    ) -> CliResult {
        self.commit_module_with(root, module, expected, replacement, || {}, || {})
    }

    pub(super) fn commit_module_with(
        &self,
        root: &Path,
        module: &str,
        expected: &str,
        replacement: &str,
        before_final_check: impl FnOnce(),
        after_publish: impl FnOnce(),
    ) -> CliResult {
        self.revalidate(root)?;
        let label = root.join(module);
        let parent = path::resolve(&self.directory, module, &label)?;
        let original = path::read_target(&parent, &label, expected.len())?;
        require_source(&label, &original.bytes, expected.as_bytes())?;
        let staged = Staged::create(
            &parent.descriptor,
            original.mode,
            replacement.as_bytes(),
            &label,
        )?;
        before_final_check();
        self.revalidate(root)?;
        path::require_parent(&self.directory, module, parent.identity, &label)?;
        let current = path::read_target(&parent, &label, expected.len())?;
        if current.identity != original.identity || current.mode != original.mode {
            return Err(changed(
                &label,
                "module identity or mode changed before commit",
            ));
        }
        require_source(&label, &current.bytes, expected.as_bytes())?;
        staged.publish(&parent, &label)?;
        after_publish();
        self.revalidate(root).map_err(committed)?;
        path::require_parent(&self.directory, module, parent.identity, &label).map_err(committed)
    }
}

fn require_source(label: &Path, actual: &[u8], expected: &[u8]) -> CliResult {
    if actual == expected {
        Ok(())
    } else {
        Err(changed(label, "module bytes changed before commit"))
    }
}

fn changed(label: &Path, message: &str) -> CliError {
    CliError::new(
        "SOURCE_CHANGED",
        format!(
            "source {} changed during the edit: {message}",
            label.display()
        ),
    )
}

fn committed(error: CliError) -> CliError {
    CliError::new(
        "WRITE_COMMIT_UNCERTAIN",
        format!("{error}; the source replacement already committed"),
    )
}
