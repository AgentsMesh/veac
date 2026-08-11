use super::project_build::{authored_project, TARGET};
use super::support::*;

#[test]
fn project_build_rejects_a_missing_target_source_before_scheduling() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("project.veac");
    std::fs::write(&entry, authored_project()).unwrap();
    std::fs::create_dir(temp.path().join("sources")).unwrap();
    std::fs::create_dir(temp.path().join("materials")).unwrap();
    veac()
        .args(["project", "build"])
        .arg(entry)
        .assert()
        .failure()
        .stderr(predicate::str::contains("PROJECT_BUILD_CONTRACT"));
}

#[test]
fn project_build_records_incompatible_declared_outputs() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("project.veac");
    let receipt = temp.path().join("build/receipt.json");
    let project = authored_project().replace("MediaType.Video", "MediaType.Audio");
    let sources = temp.path().join("sources");
    std::fs::create_dir(&sources).unwrap();
    std::fs::write(&entry, project).unwrap();
    std::fs::write(sources.join("main.veac"), TARGET).unwrap();
    std::fs::create_dir(temp.path().join("materials")).unwrap();
    veac()
        .args(["project", "build"])
        .arg(entry)
        .arg("--receipt")
        .arg(&receipt)
        .assert()
        .failure()
        .stderr(predicate::str::contains("PROJECT_BUILD_FAILED"));
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(receipt).unwrap()).unwrap();
    assert_eq!(value["outcome"], "build_failed");
    assert!(value["nodes"][0]["message"]
        .as_str()
        .unwrap()
        .contains("kind differs from target deliverable"));
}

#[test]
fn project_build_requires_output_logical_keys_to_match() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("project.veac");
    let receipt = temp.path().join("build/receipt.json");
    let project =
        authored_project().replace("identifier(\"video\")", "identifier(\"renamed-video\")");
    let sources = temp.path().join("sources");
    std::fs::create_dir(&sources).unwrap();
    std::fs::write(&entry, project).unwrap();
    std::fs::write(sources.join("main.veac"), TARGET).unwrap();
    std::fs::create_dir(temp.path().join("materials")).unwrap();
    veac()
        .args(["project", "build"])
        .arg(entry)
        .arg("--receipt")
        .arg(&receipt)
        .assert()
        .failure();
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(receipt).unwrap()).unwrap();
    assert!(value["nodes"][0]["message"]
        .as_str()
        .unwrap()
        .contains("omitted project output logical key 'renamed-video'"));
}

#[test]
fn project_build_records_target_inputs_missing_from_the_manifest() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("project.veac");
    let receipt = temp.path().join("build/receipt.json");
    let target = format!("input parameter undeclared: text;\n{TARGET}");
    let sources = temp.path().join("sources");
    std::fs::create_dir(&sources).unwrap();
    std::fs::write(&entry, authored_project()).unwrap();
    std::fs::write(sources.join("main.veac"), target).unwrap();
    std::fs::create_dir(temp.path().join("materials")).unwrap();
    let output = veac()
        .args(["project", "build"])
        .arg(entry)
        .arg("--receipt")
        .arg(&receipt)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(receipt).unwrap()).unwrap();
    assert!(value["nodes"][0]["message"]
        .as_str()
        .unwrap()
        .contains("absent from the project target"));
}

#[test]
fn project_receipt_cannot_replace_a_target_source() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("project.veac");
    let sources = temp.path().join("sources");
    let target = sources.join("main.veac");
    std::fs::create_dir(&sources).unwrap();
    std::fs::create_dir(temp.path().join("materials")).unwrap();
    std::fs::write(&entry, authored_project()).unwrap();
    std::fs::write(&target, TARGET).unwrap();

    veac()
        .args(["project", "build"])
        .arg(&entry)
        .arg("--receipt")
        .arg(&target)
        .assert()
        .failure()
        .stderr(predicate::str::contains("OUTPUT_OVERWRITES_INPUT"));
    assert_eq!(std::fs::read_to_string(target).unwrap(), TARGET);
}
