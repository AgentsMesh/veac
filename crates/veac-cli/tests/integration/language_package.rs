use super::language_package_fixture as fixture;
use super::support::*;

#[test]
fn package_api_prints_the_signed_canonical_metadata() {
    let temp = tempdir().unwrap();
    let root = temp.path().join("root");
    fixture::write_exact_package(&root, "editor-kit", "1.0.0", true);

    let output = veac().args(["package", "api"]).arg(&root).output().unwrap();

    assert_success(&output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout,
        std::fs::read_to_string(root.join("veac.package.api.json")).unwrap()
    );
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(
        serde_json_canonicalizer::to_string(&value).unwrap(),
        stdout.trim()
    );
}

#[test]
fn package_inspect_reports_root_dependency_and_content_digests() {
    let temp = tempdir().unwrap();
    let root = temp.path().join("root");
    let signed = fixture::write_exact_package(&root, "editor-kit", "1.0.0", true);

    let output = veac()
        .args(["package", "inspect"])
        .arg(&root)
        .output()
        .unwrap();

    assert_success(&output);
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["root"]["package"]["name"], "editor-kit");
    assert_eq!(value["root"]["content_sha256"], signed.content_sha256);
    assert_digest(&value["root"]["entry_sha256"]);
    assert_eq!(value["dependencies"].as_array().unwrap().len(), 1);
    assert_eq!(
        value["dependencies"][0]["package"]["name"],
        "editor-kit-util"
    );
    assert_eq!(
        value["dependencies"][0]["content_sha256"],
        signed.dependency_content_sha256.unwrap()
    );
    assert_digest(&value["dependencies"][0]["entry_sha256"]);
}

#[test]
fn package_search_is_stable_sorted_and_ignores_plain_directories() {
    let temp = tempdir().unwrap();
    let store = temp.path().join("store");
    std::fs::create_dir(&store).unwrap();
    fixture::write_exact_package(&store.join("z-second"), "clip-tools", "2.0.0", false);
    fixture::write_exact_package(&store.join("a-first"), "clip-tools", "1.0.0", false);
    std::fs::create_dir(store.join("notes")).unwrap();
    std::fs::write(store.join("notes/README.md"), "not a VEAC package\n").unwrap();

    let first = search(&store, "clip");
    let second = search(&store, "clip");

    assert_eq!(first, second);
    let value: serde_json::Value = serde_json::from_slice(&first).unwrap();
    let packages = value["packages"].as_array().unwrap();
    assert_eq!(packages.len(), 2);
    assert_eq!(packages[0]["package"]["version"], "1.0.0");
    assert_eq!(packages[1]["package"]["version"], "2.0.0");
}

#[test]
fn package_tampering_returns_the_stable_digest_diagnostic() {
    let temp = tempdir().unwrap();
    let root = temp.path().join("root");
    fixture::write_exact_package(&root, "editor-kit", "1.0.0", false);
    std::fs::write(root.join("main.veac"), "module { }\n").unwrap();

    veac()
        .args(["package", "inspect"])
        .arg(&root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("LANG_PACKAGE_DIGEST"));
}

#[test]
fn package_search_rejects_non_canonical_queries() {
    let temp = tempdir().unwrap();

    veac()
        .args(["package", "search"])
        .arg(temp.path())
        .arg("Clip Tools")
        .assert()
        .failure()
        .stderr(predicate::str::contains("LANG_PACKAGE_QUERY"));
}

#[test]
fn bundle_owns_delivery_packaging_and_package_requires_an_operation() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, GENERATED_SOURCE);
    let destination = temp.path().join("bundle");

    veac()
        .arg("bundle")
        .arg(&project)
        .args(["--destination"])
        .arg(&destination)
        .assert()
        .success()
        .stdout(predicate::str::contains("Bundled:"));
    assert!(destination.join("project.veac.json").is_file());
    veac()
        .arg("package")
        .arg(&project)
        .args(["--destination"])
        .arg(temp.path().join("old-package"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage: veac package"));
}

fn search(store: &std::path::Path, query: &str) -> Vec<u8> {
    let output = veac()
        .args(["package", "search"])
        .arg(store)
        .arg(query)
        .output()
        .unwrap();
    assert_success(&output);
    output.stdout
}

fn assert_success(output: &std::process::Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_digest(value: &serde_json::Value) {
    let value = value.as_str().unwrap();
    assert_eq!(value.len(), 64);
    assert!(value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
}
