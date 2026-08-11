use super::select_config;

fn envelope() -> veac_ir::ProjectEnvelope {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../veac-ir/tests/fixtures/minimal-project.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn config_selection_requires_exactly_one_output() {
    let mut value = envelope();
    let expected = value.project.render_configs[0].id.clone();
    assert_eq!(select_config(&value).unwrap(), expected);

    value.project.render_configs.clear();
    assert!(select_config(&value)
        .unwrap_err()
        .to_string()
        .contains("no render configuration"));

    value = envelope();
    value
        .project
        .render_configs
        .push(value.project.render_configs[0].clone());
    assert!(select_config(&value)
        .unwrap_err()
        .to_string()
        .contains("exactly one"));
}
