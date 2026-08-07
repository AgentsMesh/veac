use super::super::support::*;

#[test]
fn language_spec_cli_publishes_the_pinned_plugin_registry() {
    let output = veac().arg("language-spec").output().unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let plugins = value["plugin_effects"].as_array().unwrap();
    assert_eq!(plugins.len(), veac_ir::plugin_effects().len());
    let plugin = &plugins[0];
    assert_eq!(
        plugin["descriptor_constructor"],
        "plugin_reference_monochrome_v1"
    );
    assert_eq!(
        plugin["application_constructor"],
        "video_plugin_scalar_effect"
    );
    assert_eq!(
        plugin["effect_type"],
        veac_ir::REFERENCE_MONOCHROME_EFFECT_TYPE
    );
    assert_eq!(plugin["digest"], veac_ir::REFERENCE_MONOCHROME_DIGEST);
    assert_eq!(plugin["parameters"][0]["value_type"], "number");
    assert_eq!(plugin["parameters"][0]["minimum"], "0");
    assert_eq!(plugin["parameters"][0]["maximum"], "1");
    assert_eq!(plugin["parameters"][0]["supports_curve"], true);
    assert_eq!(plugin["determinism"], "deterministic");
    assert_eq!(
        plugin["supported_backends"],
        serde_json::json!(["ffmpeg-8"])
    );
}

#[test]
fn language_spec_schema_exposes_strict_plugin_descriptor_types() {
    let output = veac().args(["language-spec", "--schema"]).output().unwrap();
    assert!(output.status.success());
    let schema: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(schema["properties"]["plugin_effects"]["uniqueItems"], true);
    assert_eq!(
        schema["$defs"]["PluginEffectSpec"]["additionalProperties"],
        false
    );
    assert_eq!(
        schema["$defs"]["PluginEffectSpec"]["properties"]["digest"]["pattern"],
        "^[0-9a-f]{64}$"
    );
    assert_eq!(
        schema["$defs"]["PluginParameterSpec"]["additionalProperties"],
        false
    );
}
