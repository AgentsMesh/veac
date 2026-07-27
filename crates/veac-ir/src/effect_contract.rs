#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectDomain {
    Video,
    Audio,
}

pub fn effect_domain(effect_type: &str) -> Option<EffectDomain> {
    crate::built_in_effect(effect_type)?;
    if effect_type.starts_with("video.") {
        Some(EffectDomain::Video)
    } else if effect_type.starts_with("audio.") {
        Some(EffectDomain::Audio)
    } else {
        None
    }
}
