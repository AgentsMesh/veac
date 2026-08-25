use super::language_package_fixture as package_contract_fixture;
use super::support::*;
#[path = "source_package/fixture.rs"]
mod fixture;
use fixture::*;

#[path = "source_package/failures.rs"]
mod failures;
#[path = "source_package/split_brain.rs"]
mod split_brain;

#[test]
fn build_and_check_use_the_explicit_verified_package_root() {
    let temp = tempdir().unwrap();
    let source = project(&temp);
    let package = standard_package();
    veac()
        .args(["build", source.to_str().unwrap(), "--package-root"])
        .arg(&package)
        .assert()
        .success()
        .stdout(predicate::str::contains("cli-e2e"));
    veac()
        .args(["check", source.to_str().unwrap(), "--package-root"])
        .arg(&package)
        .assert()
        .success()
        .stdout(predicate::str::contains("Executable source is valid"));
}

#[test]
fn exact_package_import_fails_closed_without_a_mount() {
    let temp = tempdir().unwrap();
    let source = project(&temp);
    veac()
        .args(["build", source.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("PROGRAM_IMPORT_LOAD"));
}

#[test]
fn fmt_revision_and_index_share_the_same_package_contract() {
    let temp = tempdir().unwrap();
    let source = project(&temp);
    let package = standard_package();
    veac()
        .args([
            "fmt",
            source.to_str().unwrap(),
            "--stdout",
            "--package-root",
        ])
        .arg(&package)
        .assert()
        .success();
    let revision = veac()
        .args([
            "source-revision",
            source.to_str().unwrap(),
            "--package-root",
        ])
        .arg(&package)
        .output()
        .unwrap();
    assert!(revision.status.success());
    let index = veac()
        .args(["source-index", source.to_str().unwrap(), "--package-root"])
        .arg(&package)
        .output()
        .unwrap();
    assert!(index.status.success());
    let revision: serde_json::Value = serde_json::from_slice(&revision.stdout).unwrap();
    let index: serde_json::Value = serde_json::from_slice(&index.stdout).unwrap();
    assert_eq!(index["revision"], revision);
    let modules = index["modules"].as_array().unwrap();
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0]["module"], "main.veac");
}

#[test]
fn source_edit_dry_run_and_publish_recompile_against_the_package_fallback() {
    let temp = tempdir().unwrap();
    let source = project(&temp);
    let package = standard_package();
    let batch_path = temp.path().join("package-edit.json");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&edit_batch(&source, &package))
            .unwrap(),
    )
    .unwrap();
    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
            "--dry-run",
            "--package-root",
        ])
        .arg(&package)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dry_run\": true"));
    assert!(std::fs::read_to_string(&source)
        .unwrap()
        .contains("{ 200ms }"));
    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
            "--package-root",
        ])
        .arg(&package)
        .assert()
        .success();
    assert!(std::fs::read_to_string(source)
        .unwrap()
        .contains("{ 400ms }"));
}
