use super::*;

#[test]
fn content_addressed_plugin_emits_registered_ffmpeg_adapter() {
    let mut plan = resolved(&fixture());
    clip(&mut plan).effects = vec![effect("fx_plugin_monochrome", plugin(constant(0.75)))];
    let graph = graph(&plan);
    assert!(graph.contains("hue=s='1-(0.75)'"), "{graph}");
    assert!(graph.contains("enable='gte(t,0)*lt(t,1)'"), "{graph}");
}

#[test]
fn plugin_backend_requires_the_exact_typed_parameter_schema() {
    let value = effect("fx_plugin_schema", plugin(constant(0.5)));
    let mut missing = serde_json::to_value(&value).unwrap();
    missing["effect"].as_object_mut().unwrap().remove("amount");
    assert!(serde_json::from_value::<ResolvedEffect>(missing).is_err());
    let mut extra = serde_json::to_value(value).unwrap();
    extra["effect"]["backend_flag"] = serde_json::json!(true);
    assert!(serde_json::from_value::<ResolvedEffect>(extra).is_err());
}

#[test]
fn plugin_amount_supports_temporal_number_curves() {
    let mut plan = resolved(&fixture());
    clip(&mut plan).effects = vec![effect("fx_plugin_curve", plugin(curve(1.0)))];
    let graph = graph(&plan);
    assert!(graph.contains("hue=s='1-(if("), "{graph}");
    assert!(graph.contains("clip((t-0)/1\\,0\\,1)"), "{graph}");
}

fn plugin(amount: Animatable<f64>) -> Effect {
    Effect::VideoPluginReferenceMonochromeV1 {
        descriptor_digest: PluginEffectDigest::new(REFERENCE_MONOCHROME_DIGEST).unwrap(),
        amount,
    }
}
