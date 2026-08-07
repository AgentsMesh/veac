use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn effect(
        &mut self,
        effect: &EffectInstance,
        duration: RationalTime,
        timebase: u32,
        path: &str,
        item_id: &str,
    ) {
        let effect_path = format!("{path}/effects/{}", effect.id);
        self.check_id(effect.id.is_valid(), effect.id.as_str(), &effect_path);
        if !self.effect_ids.insert(effect.id.to_string()) {
            self.duplicate("DUPLICATE_EFFECT_ID", effect.id.as_str(), &effect_path);
        }
        if let Some(range) = effect.enable_range {
            self.time_range(
                range,
                timebase,
                "EFFECT_RANGE",
                &format!("{effect_path}/enable_range"),
                item_id,
            );
            if range.end().is_ok_and(|end| end > duration) {
                self.value_error("EFFECT_RANGE", &effect_path, item_id);
            }
        }
        self.effect_parameters(effect, duration, timebase, &effect_path, item_id);
        self.plugin_effect(effect, &effect_path, item_id);
    }

    fn effect_parameters(
        &mut self,
        effect: &EffectInstance,
        duration: RationalTime,
        timebase: u32,
        path: &str,
        item_id: &str,
    ) {
        let specification = registered_effect(effect.kind()).expect("closed effect registry");
        for parameter in specification.parameters {
            let Some(value) = effect.effect.parameter(parameter.parameter) else {
                self.value_error("EFFECT_PARAMETER_SCHEMA", path, item_id);
                continue;
            };
            if !parameter_matches(*parameter, value) {
                self.value_error("EFFECT_PARAMETER_RANGE", path, item_id);
            }
            if let EffectParameterRef::Curve(value) = value {
                self.animatable(
                    value,
                    duration,
                    timebase,
                    &format!("{path}/effect/{}", parameter.parameter.name()),
                    item_id,
                    |number| number.is_finite(),
                );
            }
        }
    }

    fn plugin_effect(&mut self, effect: &EffectInstance, path: &str, item_id: &str) {
        let Effect::VideoPluginReferenceMonochromeV1 {
            descriptor_digest, ..
        } = &effect.effect
        else {
            return;
        };
        let descriptor = plugin_effect(effect.kind()).expect("closed plugin effect registry");
        if !descriptor_digest.is_valid()
            || descriptor_digest.as_str() != descriptor.digest
            || !descriptor.digest_matches()
        {
            self.value_error("PLUGIN_EFFECT_DESCRIPTOR_DIGEST", path, item_id);
        }
    }
}
