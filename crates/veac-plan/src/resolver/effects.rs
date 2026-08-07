use veac_ir::{Clip, EffectDomain, RationalTime, TimeRange};

use crate::ResolvedEffect;

pub(super) fn resolve_effects(clip: &Clip, video: bool, audio: bool) -> Vec<ResolvedEffect> {
    clip.effects
        .iter()
        .filter(|effect| {
            effect.enabled
                && match effect.domain() {
                    EffectDomain::Video => video,
                    EffectDomain::Audio => audio,
                }
        })
        .map(|effect| ResolvedEffect {
            id: effect.id.clone(),
            active_range: effect.enable_range.unwrap_or(TimeRange {
                start: RationalTime {
                    value: 0,
                    timescale: clip.record_range.duration.timescale,
                },
                duration: clip.record_range.duration,
            }),
            effect: effect.effect.clone(),
        })
        .collect()
}
