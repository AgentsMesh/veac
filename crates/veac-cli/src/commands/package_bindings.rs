use std::path::{Path, PathBuf};

use crate::arguments::PackageBindingsArgs;
use crate::{CliError, CliResult};

pub(crate) fn run(arguments: PackageBindingsArgs) -> CliResult {
    let root = canonical_directory(&arguments.package)?;
    reject_package_output(&root, arguments.output.as_deref())?;
    let manifest_file = crate::fs::canonical_file(&root.join("package.json"), "package manifest")?;
    let manifest: veac_artifact::PackageManifest =
        super::workflow_io::read_json(&manifest_file, "package manifest")?;
    let project = veac_artifact::packaged_project(&root, &manifest)
        .map_err(|error| CliError::new("PACKAGE_BINDINGS_FAILED", error.to_string()))?;
    let bindings = veac_artifact::package_binding_manifest(&root, &manifest)
        .map_err(|error| CliError::new("PACKAGE_BINDINGS_FAILED", error.to_string()))?;
    let bytes = veac_artifact::canonical_binding_bytes(&bindings)
        .map_err(|error| CliError::new("PACKAGE_BINDINGS_FAILED", error.to_string()))?;
    let mut protected = vec![manifest_file, project];
    protected.extend(
        manifest
            .entries
            .iter()
            .map(|entry| root.join(&entry.packaged_path)),
    );
    super::workflow_io::write(&bytes, arguments.output.as_deref(), &protected)
}

fn reject_package_output(root: &Path, output: Option<&Path>) -> CliResult {
    let Some(output) = output else {
        return Ok(());
    };
    let parent = std::fs::canonicalize(output.parent().unwrap_or(Path::new(".")))
        .map_err(|error| CliError::new("PACKAGE_BINDINGS_FAILED", error.to_string()))?;
    if parent.starts_with(root) {
        Err(CliError::new(
            "PACKAGE_BINDINGS_OUTPUT_CONFLICT",
            "execution bindings must be written outside the portable package",
        ))
    } else {
        Ok(())
    }
}

fn canonical_directory(path: &Path) -> CliResult<PathBuf> {
    let authored = std::fs::symlink_metadata(path)
        .map_err(|error| CliError::new("PACKAGE_BINDINGS_FAILED", error.to_string()))?;
    if authored.file_type().is_symlink() || !authored.is_dir() {
        return Err(CliError::new(
            "PACKAGE_BINDINGS_FAILED",
            "package root must be a non-symlink directory",
        ));
    }
    let canonical = std::fs::canonicalize(path).map_err(|error| {
        CliError::new(
            "PACKAGE_BINDINGS_FAILED",
            format!("cannot resolve package {}: {error}", path.display()),
        )
    })?;
    let metadata = std::fs::symlink_metadata(&canonical)
        .map_err(|error| CliError::new("PACKAGE_BINDINGS_FAILED", error.to_string()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(CliError::new(
            "PACKAGE_BINDINGS_FAILED",
            "package root must be a non-symlink directory",
        ));
    }
    Ok(canonical)
}
