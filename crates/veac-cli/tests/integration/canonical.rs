use super::support::*;

#[test]
fn schema_and_plan_emit_parseable_contracts_without_local_paths() {
    let schema = veac().arg("schema").output().unwrap();
    assert!(schema.status.success());
    let schema: serde_json::Value = serde_json::from_slice(&schema.stdout).unwrap();
    assert_eq!(schema["title"], "ProjectEnvelope");
    let provider = veac()
        .args(["schema", "--contract", "provider-request"])
        .output()
        .unwrap();
    assert!(provider.status.success());
    let provider: serde_json::Value = serde_json::from_slice(&provider.stdout).unwrap();
    assert_eq!(provider["title"], "ProviderRequestEnvelope");

    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, GENERATED_SOURCE);
    let plan = veac()
        .args(["plan", project.to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();
    assert!(
        plan.status.success(),
        "{}",
        String::from_utf8_lossy(&plan.stderr)
    );
    let plan: serde_json::Value = serde_json::from_slice(&plan.stdout).unwrap();
    assert_eq!(
        plan["header"]["schema_version"],
        veac_plan::CURRENT_RENDER_PLAN_VERSION
    );
    assert_eq!(plan["output"]["render_config_id"], "out_main");
    assert!(!plan
        .to_string()
        .contains(&temp.path().display().to_string()));
}

#[test]
fn multiple_outputs_require_a_typed_config_selection() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, GENERATED_SOURCE);
    let mut envelope =
        veac_ir::decode_canonical_json(&std::fs::read_to_string(&project).unwrap()).unwrap();
    let mut second = envelope.project.render_configs[0].clone();
    second.id = veac_ir::RenderConfigId::new("out_second").unwrap();
    second.deliverables[0].id = veac_ir::DeliverableId::new("dlv_second").unwrap();
    second.deliverables[0].file_name = "second.mp4".into();
    envelope.project.render_configs.push(second);
    std::fs::write(&project, veac_ir::canonical_json(&envelope).unwrap()).unwrap();

    veac()
        .args(["plan", project.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("RENDER_CONFIG_REQUIRED"));
    veac()
        .args(["plan", project.to_str().unwrap(), "--config", "out_second"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"render_config_id\":\"out_second\"",
        ));
}

#[test]
fn remote_materials_fail_closed_before_render_planning() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, MEDIA_SOURCE);
    let mut envelope =
        veac_ir::decode_canonical_json(&std::fs::read_to_string(&project).unwrap()).unwrap();
    envelope.project.materials[0].source = veac_ir::MaterialSource::Remote {
        uri: "https://example.test/clip.mp4".into(),
    };
    std::fs::write(&project, veac_ir::canonical_json(&envelope).unwrap()).unwrap();
    veac()
        .args(["plan", project.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("REMOTE_MATERIAL_UNRESOLVED"));
}
