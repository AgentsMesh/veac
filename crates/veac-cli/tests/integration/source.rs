use super::support::*;

#[test]
fn help_and_source_commands_expose_the_authoring_pipeline() {
    veac()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("compile"))
        .stdout(predicate::str::contains("check-ir"))
        .stdout(predicate::str::contains("render"));

    let temp = tempdir().unwrap();
    let source = source_file(&temp, GENERATED_SOURCE);
    veac()
        .args(["check", source.to_str().unwrap(), "--revision", "12"])
        .assert()
        .success()
        .stdout(predicate::str::contains("revision 12"));

    let output = veac()
        .args(["compile", source.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], veac_ir::CURRENT_SCHEMA_VERSION);
    assert_eq!(value["project"]["id"], "prj_cli-e2e");
}

#[test]
fn compile_and_check_ir_form_a_strict_disk_round_trip() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, GENERATED_SOURCE);
    veac()
        .args(["check-ir", project.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Canonical IR is valid"));

    let mut value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&project).unwrap()).unwrap();
    value["unknown"] = true.into();
    std::fs::write(&project, serde_json::to_string(&value).unwrap()).unwrap();
    veac()
        .args(["check-ir", project.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("CANONICAL_JSON"))
        .stderr(predicate::str::contains("unknown field"));
}

#[test]
fn formatter_supports_check_stdout_and_in_place_modes() {
    let temp = tempdir().unwrap();
    let compact = GENERATED_SOURCE.replace("  ", " ");
    let source = source_file(&temp, &compact);
    veac()
        .args(["fmt", source.to_str().unwrap(), "--check"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("FORMAT_REQUIRED"));
    veac()
        .args(["fmt", source.to_str().unwrap(), "--stdout"])
        .assert()
        .success()
        .stdout(predicate::str::contains("project cli-e2e {"));
    veac()
        .args(["fmt", source.to_str().unwrap()])
        .assert()
        .success();
    veac()
        .args(["fmt", source.to_str().unwrap(), "--check"])
        .assert()
        .success();
}

#[test]
fn authoring_diagnostics_are_stable_located_and_json_serializable() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, "project broken {\n  unsupported value;\n}");
    veac()
        .args(["check", source.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error[AUTHORING_"))
        .stderr(predicate::str::contains("main.veac:2:"));

    let output = veac()
        .args([
            "--diagnostic-format",
            "json",
            "check",
            source.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let envelope: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert!(envelope["diagnostics"][0]["code"]
        .as_str()
        .unwrap()
        .starts_with("AUTHORING_"));
    assert_eq!(envelope["diagnostics"][0]["source_span"]["line"], 2);
}

#[test]
fn canonical_diagnostics_preserve_agent_repair_context() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, MEDIA_SOURCE);
    let mut envelope =
        veac_ir::decode_canonical_json(&std::fs::read_to_string(&project).unwrap()).unwrap();
    envelope
        .project
        .materials
        .push(envelope.project.materials[0].clone());
    std::fs::write(&project, serde_json::to_vec(&envelope).unwrap()).unwrap();
    let output = veac()
        .args(["check-ir", project.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[DUPLICATE_MATERIAL_ID]"));
    assert!(stderr.contains("object: med_footage"));
    assert!(stderr.contains("/project/materials/med_footage"));
}
