use super::support::*;

#[path = "project_derivation/matrix.rs"]
mod matrix;

const PROJECT: &str = include_str!("../../../veac-project/tests/fixtures/authored_derivation.veac");

#[test]
fn project_build_executes_and_caches_a_real_media_derivation() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("project.veac");
    let receipt = temp.path().join("build/receipt.json");
    std::fs::create_dir(temp.path().join("sources")).unwrap();
    let materials = temp.path().join("materials");
    std::fs::create_dir(&materials).unwrap();
    std::fs::write(&entry, PROJECT).unwrap();
    make_video(&materials.join("source.mp4"));

    let first = invoke(&entry, &receipt);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let output = temp.path().join("dist/proxy.mp4");
    assert_eq!(video_dimensions(&output), "16x16");
    assert_eq!(receipt_status(&receipt), "executed");

    let second = invoke(&entry, &receipt);
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(receipt_status(&receipt), "cache_hit");
    assert_eq!(video_dimensions(&output), "16x16");
}

fn invoke(entry: &std::path::Path, receipt: &std::path::Path) -> std::process::Output {
    veac()
        .args(["project", "build"])
        .arg(entry)
        .arg("--receipt")
        .arg(receipt)
        .output()
        .unwrap()
}

fn receipt_status(receipt: &std::path::Path) -> String {
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(receipt).unwrap()).unwrap();
    value["nodes"][0]["status"].as_str().unwrap().to_owned()
}
