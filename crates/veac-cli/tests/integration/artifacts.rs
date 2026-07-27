use super::support::*;

#[test]
fn manifest_is_machine_neutral_and_output_is_canonical() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, GENERATED_SOURCE);
    let output = temp.path().join("build.json");
    veac()
        .args([
            "manifest",
            project.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();
    let bytes = std::fs::read(&output).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["schema"], "https://veac.dev/schemas/build-manifest");
    assert_eq!(value["output"]["render_config_id"], "out_main");
    assert!(!String::from_utf8(bytes)
        .unwrap()
        .contains(&temp.path().display().to_string()));
}

#[test]
fn package_contains_only_reachable_identity_verified_inputs() {
    let temp = tempdir().unwrap();
    make_video(&temp.path().join("clip.mp4"));
    let project = compile_ir(&temp, MEDIA_SOURCE);
    let package = temp.path().join("bundle");
    veac()
        .args([
            "package",
            project.to_str().unwrap(),
            "--destination",
            package.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Packaged:"));
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(package.join("package.json")).unwrap()).unwrap();
    assert_eq!(manifest["entries"].as_array().unwrap().len(), 1);
    let relative = manifest["entries"][0]["packaged_path"].as_str().unwrap();
    assert!(package.join(relative).is_file());
    assert!(package.join("project.veac.json").is_file());
}

#[test]
fn package_bindings_restore_render_without_the_original_media() {
    let temp = tempdir().unwrap();
    let media = temp.path().join("clip.mp4");
    make_video(&media);
    let project = compile_ir(&temp, MEDIA_SOURCE);
    let package = temp.path().join("bundle");
    veac()
        .args([
            "package",
            project.to_str().unwrap(),
            "--destination",
            package.to_str().unwrap(),
        ])
        .assert()
        .success();
    std::fs::remove_file(media).unwrap();
    let bindings = temp.path().join("bindings.json");
    let conflicting_bindings = package.join("bindings.json");
    veac()
        .args([
            "package-bindings",
            package.to_str().unwrap(),
            "--output",
            conflicting_bindings.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("PACKAGE_BINDINGS_OUTPUT_CONFLICT"));
    veac()
        .args([
            "package-bindings",
            package.to_str().unwrap(),
            "--output",
            bindings.to_str().unwrap(),
        ])
        .assert()
        .success();
    let packaged_project = package.join("project.veac.json");
    let restored_manifest = temp.path().join("restored-manifest.json");
    veac()
        .args([
            "plan",
            packaged_project.to_str().unwrap(),
            "--bindings",
            bindings.to_str().unwrap(),
        ])
        .assert()
        .success();
    veac()
        .args([
            "manifest",
            packaged_project.to_str().unwrap(),
            "--bindings",
            bindings.to_str().unwrap(),
            "--output",
            restored_manifest.to_str().unwrap(),
        ])
        .assert()
        .success();
    let output = temp.path().join("restored-output");
    std::fs::create_dir(&output).unwrap();
    veac()
        .args([
            "render",
            packaged_project.to_str().unwrap(),
            "--bindings",
            bindings.to_str().unwrap(),
            "--destination",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert!(output.join("media-render.mp4").is_file());
}

#[test]
fn relink_discovers_exact_identity_and_rejects_ambiguity() {
    let temp = tempdir().unwrap();
    let media = temp.path().join("clip.mp4");
    make_video(&media);
    let project = compile_ir(&temp, MEDIA_SOURCE);
    let package = temp.path().join("bundle");
    veac()
        .args([
            "package",
            project.to_str().unwrap(),
            "--destination",
            package.to_str().unwrap(),
        ])
        .assert()
        .success();
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(package.join("package.json")).unwrap()).unwrap();
    let packaged = package.join(manifest["entries"][0]["packaged_path"].as_str().unwrap());
    let search = temp.path().join("search");
    std::fs::create_dir(&search).unwrap();
    std::fs::copy(&packaged, search.join("replacement.mp4")).unwrap();
    let bindings = temp.path().join("relinked.json");
    veac()
        .args([
            "relink",
            package.join("project.veac.json").to_str().unwrap(),
            "--search",
            search.to_str().unwrap(),
            "--output",
            bindings.to_str().unwrap(),
        ])
        .assert()
        .success();
    std::fs::copy(&packaged, search.join("duplicate.mp4")).unwrap();
    veac()
        .args([
            "relink",
            package.join("project.veac.json").to_str().unwrap(),
            "--search",
            search.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("RELINK_INCOMPLETE"));
}
