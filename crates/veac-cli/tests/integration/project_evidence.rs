use super::support::*;

const PROJECT: &str = include_str!("fixtures/project_evidence/project.veac");
const TARGET: &str = include_str!("fixtures/project_evidence/main.veac");
const PASS: &str = include_str!("fixtures/project_evidence/evidence-pass.veac");
const FAIL: &str = include_str!("fixtures/project_evidence/evidence-fail.veac");

#[test]
fn project_evidence_publishes_failed_assertions_without_losing_the_bundle() {
    let fixture = fixture(FAIL);
    let receipt = fixture.path().join("build/evidence-receipt.json");
    veac()
        .args(["project", "evidence"])
        .arg(fixture.path().join("project.veac"))
        .arg("--receipt")
        .arg(&receipt)
        .assert()
        .success();

    let bundle = fixture.path().join("dist/evidence/preview/bundle.json");
    let results = fixture.path().join("dist/evidence/preview/results.json");
    assert_eq!(json(&bundle)["outcome"], "fail");
    assert_eq!(json(&results)["outcome"], "fail");
    assert_eq!(json(&receipt)["outcome"], "succeeded");

    veac()
        .args(["project", "test"])
        .arg(fixture.path().join("project.veac"))
        .arg("--receipt")
        .arg(fixture.path().join("build/test-receipt.json"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("PROJECT_TEST_FAILED"));
    assert!(results.is_file());
}

#[test]
fn project_test_passes_and_reuses_a_typed_evidence_bundle() {
    let fixture = fixture(PASS);
    let entry = fixture.path().join("project.veac");
    for receipt in ["first.json", "second.json"] {
        veac()
            .args(["project", "test"])
            .arg(&entry)
            .arg("--receipt")
            .arg(fixture.path().join("build").join(receipt))
            .assert()
            .success();
    }
    let second = json(&fixture.path().join("build/second.json"));
    assert!(second["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .all(|node| node["status"] == "cache_hit"));
    assert_eq!(
        json(&fixture.path().join("dist/evidence/preview/bundle.json"))["outcome"],
        "pass"
    );
}

fn fixture(evidence: &str) -> TempDir {
    let temp = tempdir().unwrap();
    let sources = temp.path().join("sources");
    std::fs::create_dir(&sources).unwrap();
    std::fs::write(temp.path().join("project.veac"), PROJECT).unwrap();
    std::fs::write(sources.join("main.veac"), TARGET).unwrap();
    std::fs::write(sources.join("evidence.veac"), evidence).unwrap();
    std::fs::create_dir(temp.path().join("materials")).unwrap();
    temp
}

fn json(path: &std::path::Path) -> serde_json::Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
