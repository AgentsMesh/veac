use veac_ir::{effect_domain, Clip, EffectDomain, RationalTime, TimeRange};

use crate::ResolvedEffect;

pub(super) fn resolve_effects(clip: &Clip, video: bool, audio: bool) -> Vec<ResolvedEffect> {
    clip.effects
        .iter()
        .filter(|effect| {
            effect.enabled
                && match effect_domain(&effect.effect_type) {
                    Some(EffectDomain::Video) => video,
                    Some(EffectDomain::Audio) => audio,
                    None => false,
                }
        })
        .map(|effect| ResolvedEffect {
            id: effect.id.clone(),
            effect_type: effect.effect_type.clone(),
            active_range: effect.enable_range.unwrap_or(TimeRange {
                start: RationalTime {
                    value: 0,
                    timescale: clip.record_range.duration.timescale,
                },
                duration: clip.record_range.duration,
            }),
            parameters: effect.parameters.clone(),
        })
        .collect()
}
