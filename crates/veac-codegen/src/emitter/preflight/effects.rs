use std::collections::BTreeSet;

use veac_plan::canonical::{
    parameter_matches_effect, plugin_effect, registered_effect, Effect, EffectDomain, EffectId,
    EffectKind, PluginBackend,
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
            validate_contract(check, &effect.effect, owner.clone());
            let domain_valid = match effect.effect.domain() {
                EffectDomain::Video => clip.visual.is_some(),
                EffectDomain::Audio => clip.audio.is_some(),
            };
            if !domain_valid {
                check.push(
                    "PLAN_EFFECT_DOMAIN_INVALID",
                    owner,
                    "effect domain has no corresponding resolved clip stream",
                );
            }
        }
    }
}

pub(super) fn validate_apply_effect(check: &mut Check, apply_id: &str, effect: &Effect) {
    validate_contract(check, effect, Some(apply_id.to_owned()));
    if effect.domain() != EffectDomain::Video || effect.kind() == EffectKind::VideoStabilize {
        check.push(
            "PLAN_APPLY_EFFECT_INVALID",
            Some(apply_id.to_owned()),
            "apply effect is source-dependent or has a non-video domain",
        );
    }
}

fn validate_contract(check: &mut Check, effect: &Effect, owner: Option<String>) {
    let specification = registered_effect(effect.kind()).expect("closed canonical effect registry");
    if specification
        .parameters
        .iter()
        .any(|parameter| !parameter_matches_effect(*parameter, effect))
    {
        check.push(
            "PLAN_EFFECT_PARAMETER_INVALID",
            owner.clone(),
            "effect parameter value or animation exceeds its typed contract",
        );
    }
    let Effect::VideoPluginReferenceMonochromeV1 {
        descriptor_digest, ..
    } = effect
    else {
        return;
    };
    let descriptor = plugin_effect(effect.kind()).expect("closed plugin registry");
    if descriptor_digest.as_str() != descriptor.digest
        || !descriptor.digest_matches()
        || !descriptor.supports(PluginBackend::Ffmpeg8)
    {
        check.push(
            "PLAN_PLUGIN_EFFECT_DESCRIPTOR_INVALID",
            owner,
            "plugin descriptor digest or FFmpeg 8 adapter is not pinned",
        );
    }
}
