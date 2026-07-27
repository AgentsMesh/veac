use std::collections::{BTreeMap, BTreeSet};

use veac_plan::canonical::{
    built_in_effect, effect_domain, parameter_matches, EffectDomain, EffectId, ParameterValue,
};
use veac_plan::ResolvedRenderPlan;

use super::Check;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    let mut ids = BTreeSet::new();
    for clip in plan
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
    {
        for effect in &clip.effects {
            let owner = Some(effect.id.to_string());
            if EffectId::new(effect.id.as_str()).is_err() {
                check.push(
                    "PLAN_EFFECT_ID_INVALID",
                    owner.clone(),
                    "effect ID does not satisfy the canonical identifier contract",
                );
            }
            if !ids.insert(effect.id.to_string()) {
                check.push(
                    "PLAN_EFFECT_DUPLICATE",
                    owner.clone(),
                    "effect IDs must be unique across the render plan",
                );
            }
            let Some(specification) = built_in_effect(&effect.effect_type) else {
                check.push(
                    "PLAN_EFFECT_UNKNOWN",
                    owner,
                    "effect type is absent from the canonical registry",
                );
                continue;
            };
            let domain_valid = match effect_domain(&effect.effect_type) {
                Some(EffectDomain::Video) => clip.visual.is_some(),
                Some(EffectDomain::Audio) => clip.audio.is_some(),
                None => false,
            };
            if !domain_valid {
                check.push(
                    "PLAN_EFFECT_DOMAIN_INVALID",
                    Some(effect.id.to_string()),
                    "effect domain has no corresponding resolved clip stream",
                );
            }
            for (name, value) in &effect.parameters {
                let valid = specification
                    .parameters
                    .iter()
                    .find(|parameter| parameter.name == name)
                    .is_some_and(|parameter| parameter_matches(*parameter, value));
                if !valid {
                    check.push(
                        "PLAN_EFFECT_PARAMETER_INVALID",
                        Some(effect.id.to_string()),
                        "effect parameter name, type, curve mode, or value is invalid",
                    );
                }
            }
        }
    }
}

pub(super) fn validate_apply_effect(
    check: &mut Check,
    apply_id: &str,
    effect_type: &str,
    parameters: &BTreeMap<String, ParameterValue>,
) {
    let valid = effect_domain(effect_type) == Some(EffectDomain::Video)
        && built_in_effect(effect_type).is_some_and(|specification| {
            parameters.iter().all(|(name, value)| {
                specification
                    .parameters
                    .iter()
                    .find(|parameter| parameter.name == name)
                    .is_some_and(|parameter| parameter_matches(*parameter, value))
            })
        });
    if !valid || effect_type == "video.stabilize" {
        check.push(
            "PLAN_APPLY_EFFECT_INVALID",
            Some(apply_id.to_owned()),
            "apply effect is unknown, source-dependent, or has invalid parameters",
        );
    }
}
