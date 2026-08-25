use std::path::{Path, PathBuf};

use super::project_build::{authored_project, TARGET};
use super::support::*;

const IMPORT: &str = "import \"package:veac-components@0.1.0/main.veac\" as components;\n";

#[test]
fn project_manifest_imports_require_an_explicit_package_root() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("project.veac");
    std::fs::create_dir(temp.path().join("sources")).unwrap();
    std::fs::create_dir(temp.path().join("materials")).unwrap();
    std::fs::write(&entry, format!("{IMPORT}{}", authored_project())).unwrap();

    let missing = project_command("check", &entry, None);
    assert!(!missing.status.success());
    assert!(stderr(&missing).contains("package"));

    let found = project_command("check", &entry, Some(&standard_package()));
    assert!(found.status.success(), "{}", stderr(&found));
}

#[test]
fn project_target_imports_require_the_same_explicit_package_root() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("project.veac");
    let sources = temp.path().join("sources");
    std::fs::create_dir(&sources).unwrap();
    std::fs::create_dir(temp.path().join("materials")).unwrap();
    std::fs::write(&entry, authored_project()).unwrap();
    std::fs::write(sources.join("main.veac"), format!("{IMPORT}{TARGET}")).unwrap();

    let missing = project_command("build", &entry, None);
    assert!(!missing.status.success());
    assert!(stderr(&missing).contains("package"));

    let found = project_command("build", &entry, Some(&standard_package()));
    assert!(found.status.success(), "{}", stderr(&found));
    assert!(temp
        .path()
        .join("dist/render/preview/en-us/dark.mp4")
        .is_file());
}

#[test]
fn every_project_command_rejects_package_and_writable_authority_overlap() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("project.veac");
    let sources = temp.path().join("sources");
    let package = temp.path().join("build/package");
    std::fs::create_dir(&sources).unwrap();
    std::fs::create_dir(temp.path().join("materials")).unwrap();
    std::fs::write(&entry, authored_project()).unwrap();
    std::fs::write(sources.join("main.veac"), TARGET).unwrap();
    copy_standard_package(&package);

    for command in ["check", "inspect", "graph", "build", "evidence", "test"] {
        let output = project_command(command, &entry, Some(&package));
        assert!(!output.status.success(), "{command} unexpectedly succeeded");
        assert!(stderr(&output).contains("PROJECT_PACKAGE_AUTHORITY"));
        assert!(stderr(&output).contains("build root"));
    }
}

fn project_command(command: &str, entry: &Path, package: Option<&Path>) -> std::process::Output {
    let mut process = veac();
    process.args(["project", command]).arg(entry);
    if let Some(package) = package {
        process.arg("--package-root").arg(package);
    }
    process.output().unwrap()
}

fn standard_package() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib/veac-components")
}

fn copy_standard_package(destination: &Path) {
    let source = standard_package();
    for relative in [
        "main.veac",
        "layout.veac",
        "text.veac",
        "motion.veac",
        "media.veac",
        "audio.veac",
        "delivery.veac",
        "components/card.veac",
        "veac.package.api.json",
        "veac.package.json",
        "veac.package.lock",
    ] {
        let target = destination.join(relative);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(source.join(relative), target).unwrap();
    }
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
