use veac_ir::{Animatable, Effect, REFERENCE_MONOCHROME_DIGEST};

use super::support;

fn source(amount: &str) -> String {
    let item = format!(
        "{}.with_effect(video_plugin_scalar_effect(\
         identifier(\"mono\"), effect_enabled(effect_window_full()), \
         plugin_reference_monochrome_v1(), scalar_constant({amount})))",
        support::solid("card", "#dc3f72ff", "0s", "2s")
    );
    support::visual_project(&[&item])
}

#[test]
fn typed_plugin_descriptor_lowers_to_content_addressed_canonical_effect() {
    let envelope = support::envelope(&source("0.75"));
    let effect = &support::clip(&envelope, 0).effects[0];
    assert!(matches!(
        &effect.effect,
        Effect::VideoPluginReferenceMonochromeV1 {
            descriptor_digest,
            amount,
        } if descriptor_digest.as_str() == REFERENCE_MONOCHROME_DIGEST
            && amount == &Animatable::constant(0.75)
    ));
    assert!(effect.enabled);
    assert!(effect.enable_range.is_none());
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn plugin_lowering_is_deterministic_and_range_checked() {
    assert_eq!(
        support::envelope(&source("0.5")),
        support::envelope(&source("0.5"))
    );
    let error = support::error(&source("1.01"));
    assert_eq!(error.code, "PROGRAM_EXECUTABLE_IR");
    assert!(error.message.contains("effect"));
}
