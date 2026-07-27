use crate::*;

use super::super::Validator;

impl Validator {
    pub(super) fn clip_components(
        &mut self,
        clip: &Clip,
        track_kind: TrackKind,
        timebase: u32,
        sample_rate: u32,
        path: &str,
    ) {
        if let Some(visual) = &clip.visual {
            self.visual(
                visual,
                clip.record_range.duration,
                timebase,
                path,
                clip.id.as_str(),
            );
        }
        if let Some(audio) = &clip.audio {
            self.audio(
                audio,
                clip.record_range.duration,
                timebase,
                sample_rate,
                path,
                clip.id.as_str(),
            );
        }
        for effect in &clip.effects {
            let domain_valid = match effect_domain(&effect.effect_type) {
                Some(EffectDomain::Video) => track_kind != TrackKind::Audio,
                Some(EffectDomain::Audio) => track_kind == TrackKind::Audio || clip.audio.is_some(),
                None => true,
            };
            if !domain_valid {
                self.value_error("EFFECT_MEDIA_TYPE", path, clip.id.as_str());
            }
            self.effect(
                effect,
                clip.record_range.duration,
                timebase,
                path,
                clip.id.as_str(),
            );
        }
    }
}
