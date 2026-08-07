use super::support::*;

#[test]
fn help_and_source_commands_expose_only_the_executable_pipeline() {
    veac()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("build"))
        .stdout(predicate::str::contains("compile").not())
        .stdout(predicate::str::contains("check-ir"))
        .stdout(predicate::str::contains("render"));

    let temp = tempdir().unwrap();
    let source = source_file(&temp, EXECUTABLE_SOURCE);
    veac()
        .args(["check", source.to_str().unwrap(), "--revision", "12"])
        .assert()
        .success()
        .stdout(predicate::str::contains("revision 12"));

    let output = veac()
        .args(["build", source.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], veac_ir::CURRENT_SCHEMA_VERSION);
    assert!(value["project"]["id"].as_str().unwrap().starts_with("prj_"));
}

#[test]
fn build_and_check_ir_form_a_strict_disk_round_trip() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, EXECUTABLE_SOURCE);
    let project = temp.path().join("project.json");
    veac()
        .args(["build", source.to_str().unwrap(), "--emit-ir"])
        .arg(&project)
        .assert()
        .success();
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
    let messy = EXECUTABLE_SOURCE.replacen("fn main", "fn  main", 1);
    let source = source_file(&temp, &messy);
    let expected = veac_lang::program::format_path(&source).unwrap();
    veac()
        .args(["fmt", source.to_str().unwrap(), "--check"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("FORMAT_REQUIRED"));
    veac()
        .args(["fmt", source.to_str().unwrap(), "--stdout"])
        .assert()
        .success()
        .stdout(predicate::str::contains("fn main(context: Context)"));
    assert_eq!(std::fs::read_to_string(&source).unwrap(), messy);
    veac()
        .args(["fmt", source.to_str().unwrap()])
        .assert()
        .success();
    veac()
        .args(["fmt", source.to_str().unwrap(), "--check"])
        .assert()
        .success();
    assert_eq!(std::fs::read_to_string(source).unwrap(), expected);
}

#[test]
fn executable_diagnostics_are_stable_located_and_json_serializable() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, "fn helper(value: int) -> int { value }\n");
    veac()
        .args(["check", source.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("PROGRAM_EXECUTABLE_MAIN_MISSING"))
        .stderr(predicate::str::contains("main.veac:"));

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
    assert_eq!(
        envelope["diagnostics"][0]["code"],
        "PROGRAM_EXECUTABLE_MAIN_MISSING"
    );
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
    let material_id = envelope.project.materials[0].id.to_string();
    assert!(stderr.contains("error[DUPLICATE_MATERIAL_ID]"));
    assert!(stderr.contains(&format!("object: {material_id}")));
    assert!(stderr.contains(&format!("/project/materials/{material_id}")));
}
