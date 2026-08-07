use super::super::support::predicate;
use super::support::DetachedFixture;

#[test]
fn build_publishes_detached_ir_with_an_explicit_material_root() {
    let fixture = DetachedFixture::new();
    fixture.build_success();
    let bytes = std::fs::read(&fixture.project).unwrap();
    let json = String::from_utf8(bytes).unwrap();
    let envelope = veac_ir::decode_canonical_json(&json).unwrap();
    assert_eq!(
        envelope.project.materials[0].source,
        veac_ir::MaterialSource::File {
            uri: "clip.mp4".to_owned()
        }
    );
    assert!(!json.contains(&fixture.source_root.display().to_string()));
}

#[test]
fn detached_plan_manifest_and_probe_share_the_material_root() {
    let fixture = DetachedFixture::new();
    fixture.build_success();
    let plan = fixture.command("plan").output().unwrap();
    assert_success(&plan);
    let plan_json: serde_json::Value = serde_json::from_slice(&plan.stdout).unwrap();
    assert_eq!(plan_json["inputs"][0]["canonical_uri"], "clip.mp4");
    assert!(!String::from_utf8(plan.stdout)
        .unwrap()
        .contains(&fixture.source_root.display().to_string()));

    let manifest = fixture.build_root.join("build-manifest.json");
    fixture
        .command("manifest")
        .arg("--output")
        .arg(&manifest)
        .assert()
        .success();
    let manifest_bytes = std::fs::read(&manifest).unwrap();
    assert!(!String::from_utf8(manifest_bytes)
        .unwrap()
        .contains(&fixture.source_root.display().to_string()));

    fixture
        .command("probe")
        .arg("--material")
        .arg(fixture.material_id())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema_version\""));
}

#[test]
fn detached_package_contains_verified_media_and_portable_ir() {
    let fixture = DetachedFixture::new();
    fixture.build_success();
    let package = fixture.package_root();
    fixture
        .command("package")
        .arg("--destination")
        .arg(&package)
        .assert()
        .success();
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(package.join("package.json")).unwrap()).unwrap();
    let entry = &manifest["entries"][0];
    assert!(package
        .join(entry["packaged_path"].as_str().unwrap())
        .is_file());
    let packaged_ir = std::fs::read_to_string(package.join("project.veac.json")).unwrap();
    assert!(!packaged_ir.contains(&fixture.source_root.display().to_string()));
}

fn assert_success(output: &std::process::Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
