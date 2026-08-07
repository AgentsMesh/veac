use super::*;

fn plugin_project() -> ProjectEnvelope {
    let mut project = sample_project();
    project.project.sequences[0].tracks[0].clips[0].effects[0].effect =
        Effect::VideoPluginReferenceMonochromeV1 {
            descriptor_digest: PluginEffectDigest::new(REFERENCE_MONOCHROME_DIGEST).unwrap(),
            amount: Animatable::constant(0.75),
        };
    project
}

fn codes(project: &ProjectEnvelope) -> Vec<String> {
    validate(project)
        .unwrap_err()
        .into_diagnostics()
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

#[test]
fn pinned_plugin_effect_validates_with_its_exact_typed_schema() {
    validate(&plugin_project()).unwrap();
    let value = serde_json::to_value(
        &plugin_project().project.sequences[0].tracks[0].clips[0].effects[0].effect,
    )
    .unwrap();
    let mut missing = value.clone();
    missing.as_object_mut().unwrap().remove("amount");
    assert!(serde_json::from_value::<Effect>(missing).is_err());
    let mut extra = value;
    extra["backend_flag"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Effect>(extra).is_err());
}

#[test]
fn plugin_range_and_digest_mismatches_fail_closed() {
    let mut out_of_range = plugin_project();
    let Effect::VideoPluginReferenceMonochromeV1 { amount, .. } =
        &mut out_of_range.project.sequences[0].tracks[0].clips[0].effects[0].effect
    else {
        unreachable!()
    };
    *amount = Animatable::constant(1.01);
    assert_code(&codes(&out_of_range), "EFFECT_PARAMETER_RANGE");

    let mut wrong_digest = plugin_project();
    let Effect::VideoPluginReferenceMonochromeV1 {
        descriptor_digest, ..
    } = &mut wrong_digest.project.sequences[0].tracks[0].clips[0].effects[0].effect
    else {
        unreachable!()
    };
    *descriptor_digest = serde_json::from_value(serde_json::json!("0".repeat(64))).unwrap();
    assert_code(&codes(&wrong_digest), "PLUGIN_EFFECT_DESCRIPTOR_DIGEST");
}
