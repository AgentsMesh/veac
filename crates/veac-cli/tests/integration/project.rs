use super::support::*;

const PROJECT: &str = include_str!("../../../veac-project/tests/fixtures/authored_minimal.veac");

fn fixture() -> (TempDir, std::path::PathBuf) {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("project.veac");
    std::fs::write(&entry, authored_project()).unwrap();
    (temp, entry)
}

fn invoke(name: &str, entry: &std::path::Path) -> std::process::Output {
    veac().args(["project", name]).arg(entry).output().unwrap()
}

#[test]
fn project_check_executes_and_resolves_the_workspace() {
    let (_temp, entry) = fixture();
    let output = invoke("check", &entry);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Project is valid"));
    assert!(stdout.contains("project demo"));
    assert!(stdout.contains("1 target instance(s)"));
    assert!(stdout.contains("sha256:"));
}

#[test]
fn project_inspect_prints_only_the_canonical_manifest() {
    let (_temp, entry) = fixture();
    let first = invoke("inspect", &entry);
    let second = invoke("inspect", &entry);
    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
    let value: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(value["schema"], "veac.project");
    assert_eq!(value["id"], "demo");
    assert_canonical(&first.stdout, &value);
}

#[test]
fn project_graph_prints_a_canonical_resolved_graph() {
    let (_temp, entry) = fixture();
    let output = invoke("graph", &entry);
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["version"], 1);
    assert_eq!(value["instances"].as_array().unwrap().len(), 1);
    assert_eq!(value["build_order"].as_array().unwrap().len(), 1);
    assert_canonical(&output.stdout, &value);
}

#[test]
fn project_diagnostics_preserve_language_and_contract_codes() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("project.veac");
    std::fs::write(&entry, "fn workspace() -> int { 1 }\n").unwrap();
    veac()
        .args(["project", "check"])
        .arg(&entry)
        .assert()
        .failure()
        .stderr(predicate::str::contains("PROGRAM_ENTRY_SIGNATURE"));

    let invalid = authored_project().replace("max_total_instances: 32", "max_total_instances: 0");
    std::fs::write(&entry, invalid).unwrap();
    veac()
        .args(["project", "check"])
        .arg(entry)
        .assert()
        .failure()
        .stderr(predicate::str::contains("INVALID_MATRIX"));
}

fn authored_project() -> String {
    PROJECT
        .replace("},\n    ],", "}\n    ],")
        .replace("},\n        ],", "}\n        ],")
}

fn assert_canonical(bytes: &[u8], value: &serde_json::Value) {
    let mut expected = serde_json_canonicalizer::to_vec(value).unwrap();
    expected.push(b'\n');
    assert_eq!(bytes, expected);
}
