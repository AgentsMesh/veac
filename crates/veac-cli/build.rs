#[path = "../veac-codegen/build_identity.rs"]
mod build_identity;

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use build_identity::{BuildInputs, SourceFile};

struct Inventory<'a> {
    domain: &'a str,
    output: &'a str,
    rust_roots: &'a [&'a str],
    embedded_sources: &'a [&'a str],
    manifests: &'a [&'a str],
}

fn main() {
    run().unwrap_or_else(|message| panic!("project backend build identity failed: {message}"));
}

fn run() -> Result<(), String> {
    let manifest = PathBuf::from(required("CARGO_MANIFEST_DIR")?);
    let workspace = manifest
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "veac-cli must be nested under workspace/crates".to_owned())?;
    let rustc = rustc_identity()?;
    let target = required("TARGET")?;
    let features = enabled_features();
    let version = required("CARGO_PKG_VERSION")?;
    for inventory in inventories() {
        let build = fingerprint(
            workspace, &manifest, &inventory, &rustc, &target, &features, &version,
        )?;
        println!("cargo:rustc-env={}={build}", inventory.output);
    }
    println!("cargo:rerun-if-env-changed=RUSTC");
    Ok(())
}

fn inventories() -> [Inventory<'static>; 3] {
    [
        Inventory {
            domain: "veac.project.backend",
            output: "VEAC_PROJECT_BACKEND_BUILD_SHA256",
            rust_roots: &["crates/veac-cli/src"],
            embedded_sources: &[],
            manifests: &["crates/veac-cli/Cargo.toml"],
        },
        Inventory {
            domain: "veac.project.build",
            output: "VEAC_PROJECT_BUILD_BUILD_SHA256",
            rust_roots: &[
                "crates/veac-build/src",
                "crates/veac-artifact/src",
                "crates/veac-project/src",
            ],
            embedded_sources: &["crates/veac-project/src/project.veac"],
            manifests: &[
                "crates/veac-build/Cargo.toml",
                "crates/veac-artifact/Cargo.toml",
                "crates/veac-project/Cargo.toml",
            ],
        },
        Inventory {
            domain: "veac.evidence.backend",
            output: "VEAC_EVIDENCE_BACKEND_BUILD_SHA256",
            rust_roots: &["crates/veac-evidence/src"],
            embedded_sources: &["crates/veac-evidence/src/evidence.veac"],
            manifests: &["crates/veac-evidence/Cargo.toml"],
        },
    ]
}

fn fingerprint(
    workspace: &Path,
    manifest: &Path,
    inventory: &Inventory<'_>,
    rustc: &str,
    target: &str,
    features: &[String],
    version: &str,
) -> Result<String, String> {
    let mut paths = vec![
        workspace.join("Cargo.toml"),
        workspace.join("Cargo.lock"),
        workspace.join("crates/veac-codegen/build_identity.rs"),
        manifest.join("build.rs"),
    ];
    paths.extend(inventory.manifests.iter().map(|path| workspace.join(path)));
    paths.extend(
        inventory
            .embedded_sources
            .iter()
            .map(|path| workspace.join(path)),
    );
    for root in inventory.rust_roots {
        collect_rs(&workspace.join(root), &mut paths)?;
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
    let source = build_identity::source_fingerprint(inventory.domain, &files)?;
    build_identity::build_fingerprint(
        inventory.domain,
        &BuildInputs {
            source_sha256: &source,
            rustc_verbose: rustc,
            target,
            features,
            package_version: version,
        },
    )
}

fn collect_rs(root: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|error| format!("{}: {error}", root.display()))? {
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
