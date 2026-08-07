use veac_plan::canonical::EffectParameter;
use veac_plan::{ResolvedApply, ResolvedApplyOperation, ResolvedClip};

use super::{range, text, values, Curves};
use crate::emitter::process_owner::ProcessOwner;

impl Curves<'_> {
    pub(super) fn apply(&mut self, apply: &ResolvedApply) {
        let domain = apply.record_range.duration;
        let owner = apply.id.to_string();
        let process = ProcessOwner::apply(apply);
        self.curve(
            &apply.mix.opacity,
            domain,
            &owner,
            process,
            values::unit_number,
        );
        for mask in &apply.mix.masks {
            self.curve(&mask.position, domain, &owner, process, values::unit_vec);
            self.curve(&mask.scale, domain, &owner, process, values::positive_vec);
            self.curve(
                &mask.rotation_degrees,
                domain,
                &owner,
                process,
                values::finite,
            );
            self.curve(
                &mask.feather_pixels,
                domain,
                &owner,
                process,
                values::nonnegative,
            );
            self.curve(
                &mask.expansion_pixels,
                domain,
                &owner,
                process,
                values::finite,
            );
        }
        for stage in &apply.stages {
            let ResolvedApplyOperation::Effect { effect } = &stage.operation else {
                continue;
            };
            for parameter in EffectParameter::ALL {
                if let Some(value) = effect.curve(parameter) {
                    self.curve(
                        value,
                        domain,
                        &stage.id.to_string(),
                        process,
                        values::finite,
                    );
                }
            }
        }
    }

    pub(super) fn clip(&mut self, clip: &ResolvedClip) {
        let domain = clip.record_range.duration;
        let owner = clip.id.to_string();
        let process = ProcessOwner::clip(clip);
        if let Some(visual) = &clip.visual {
            self.curve(
                &visual.transform.position,
                domain,
                &owner,
                process,
                values::point,
            );
            self.curve(
                &visual.transform.scale,
                domain,
                &owner,
                process,
                values::positive_vec,
            );
            self.curve(
                &visual.transform.rotation_degrees,
                domain,
                &owner,
                process,
                values::finite,
            );
            if let Some(crop) = &visual.transform.crop {
                self.curve(crop, domain, &owner, process, values::rect);
            }
            self.curve(
                &visual.opacity,
                domain,
                &owner,
                process,
                values::unit_number,
            );
            self.masks(&visual.masks, domain, &owner, process);
        }
        if let Some(audio) = &clip.audio {
            self.curve(&audio.gain, domain, &owner, process, values::nonnegative);
            self.curve(&audio.pan, domain, &owner, process, values::pan);
        }
        text::validate(self, clip, domain, &owner, process);
        for effect in &clip.effects {
            let effect_owner = effect.id.to_string();
            if !range::valid(effect.active_range, domain, self.timebase) {
                self.check.push(
                    "PLAN_EFFECT_RANGE_INVALID",
                    Some(effect_owner.clone()),
                    "effect range must be a valid clip-local interval",
                );
            }
            for parameter in EffectParameter::ALL {
                if let Some(value) = effect.effect.curve(parameter) {
                    self.curve(value, domain, &effect_owner, process, values::finite);
                }
            }
        }
    }

    fn masks(
        &mut self,
        masks: &[veac_plan::canonical::Mask],
        domain: veac_plan::canonical::RationalTime,
        owner: &str,
        process: ProcessOwner<'_>,
    ) {
        for mask in masks {
            self.curve(&mask.position, domain, owner, process, values::unit_vec);
            self.curve(&mask.scale, domain, owner, process, values::positive_vec);
            self.curve(
                &mask.rotation_degrees,
                domain,
                owner,
                process,
                values::finite,
            );
            self.curve(
                &mask.feather_pixels,
                domain,
                owner,
                process,
                values::nonnegative,
            );
            self.curve(
                &mask.expansion_pixels,
                domain,
                owner,
                process,
                values::finite,
            );
        }
    }
}
