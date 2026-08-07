mod build_identity;

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use build_identity::{BuildInputs, SourceFile};

const DOMAIN: &str = "veac.codegen";

fn main() {
    run().unwrap_or_else(|message| panic!("codegen build identity failed: {message}"));
}

fn run() -> Result<(), String> {
    let manifest = PathBuf::from(required("CARGO_MANIFEST_DIR")?);
    let workspace = manifest
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "veac-codegen must be nested under workspace/crates".to_owned())?;
    let mut paths = fixed_paths(workspace, &manifest);
    for relative in [
        "crates/veac-codegen/src",
        "crates/veac-plan/src",
        "crates/veac-ir/src",
    ] {
        collect_rs(&workspace.join(relative), &mut paths)?;
    }
    paths.sort();
    paths.dedup();
    let mut owned = Vec::with_capacity(paths.len());
    for path in paths {
        println!("cargo:rerun-if-changed={}", path.display());
        let relative = relative_path(workspace, &path)?;
        let bytes = fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        owned.push((relative, bytes));
    }
    let files = owned
        .iter()
        .map(|(path, bytes)| SourceFile {
            path,
            bytes: bytes.as_slice(),
        })
        .collect::<Vec<_>>();
    let source = build_identity::source_fingerprint(DOMAIN, &files)?;
    let build = build_identity::build_fingerprint(
        DOMAIN,
        &BuildInputs {
            source_sha256: &source,
            rustc_verbose: &rustc_identity()?,
            target: &required("TARGET")?,
            features: &enabled_features(),
            package_version: &required("CARGO_PKG_VERSION")?,
        },
    )?;
    println!("cargo:rerun-if-env-changed=RUSTC");
    println!("cargo:rustc-env=VEAC_CODEGEN_SOURCE_SHA256={source}");
    println!("cargo:rustc-env=VEAC_CODEGEN_BUILD_SHA256={build}");
    Ok(())
}

fn fixed_paths(workspace: &Path, manifest: &Path) -> Vec<PathBuf> {
    [
        workspace.join("Cargo.toml"),
        workspace.join("Cargo.lock"),
        workspace.join("crates/veac-ir/Cargo.toml"),
        workspace.join("crates/veac-plan/Cargo.toml"),
        manifest.join("Cargo.toml"),
        manifest.join("build.rs"),
        manifest.join("build_identity.rs"),
    ]
    .into_iter()
    .collect()
}

fn collect_rs(root: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(root).map_err(|error| format!("{}: {error}", root.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("{}: {error}", root.display()))?;
        let path = entry.path();
        let kind = entry
            .file_type()
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if kind.is_dir() {
            collect_rs(&path, output)?;
        } else if kind.is_file() && path.extension() == Some(OsStr::new("rs")) {
            output.push(path);
        }
    }
    Ok(())
}

fn relative_path(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| format!("{} is outside the workspace", path.display()))?;
    Ok(relative
        .components()
        .map(|part| part.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

fn rustc_identity() -> Result<String, String> {
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsStr::new("rustc").to_owned());
    let output = Command::new(rustc)
        .arg("-vV")
        .output()
        .map_err(|error| format!("cannot execute rustc -vV: {error}"))?;
    if !output.status.success() {
        return Err("rustc -vV failed".to_owned());
    }
    String::from_utf8(output.stdout).map_err(|_| "rustc -vV output is not UTF-8".to_owned())
}

fn enabled_features() -> Vec<String> {
    env::vars()
        .filter_map(|(name, value)| {
            (name.starts_with("CARGO_FEATURE_") && value == "1").then_some(name)
        })
        .collect()
}

fn required(name: &str) -> Result<String, String> {
    env::var(name).map_err(|_| format!("required Cargo environment `{name}` is missing"))
}
