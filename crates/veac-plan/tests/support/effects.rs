use veac_plan::canonical::*;

pub fn effect_instance(id: impl Into<String>, enabled: bool, effect: Effect) -> EffectInstance {
    EffectInstance {
        id: EffectId::new(id).unwrap(),
        enabled,
        enable_range: None,
        effect,
    }
}
