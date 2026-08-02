mod text;
mod values;

use std::collections::BTreeSet;

use veac_plan::canonical::{Animatable, Keyframe, ParameterValue, RationalTime};
use veac_plan::{ResolvedApply, ResolvedApplyOperation, ResolvedClip, ResolvedRenderPlan};

use super::Check;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    let mut curves = Curves {
        check,
        ids: BTreeSet::new(),
        timebase: plan.header.source.timebase,
    };
    for clip in plan
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
    {
        curves.clip(clip);
    }
    for apply in plan.sequences.iter().flat_map(|sequence| &sequence.applies) {
        curves.apply(apply);
    }
}

struct Curves<'a> {
    check: &'a mut Check,
    ids: BTreeSet<String>,
    timebase: u32,
}

impl Curves<'_> {
    fn apply(&mut self, apply: &ResolvedApply) {
        let domain = apply.record_range.duration;
        let owner = apply.id.to_string();
        self.curve(&apply.mix.opacity, domain, &owner, values::unit_number);
        for mask in &apply.mix.masks {
            self.curve(&mask.position, domain, &owner, values::unit_vec);
            self.curve(&mask.scale, domain, &owner, values::positive_vec);
            self.curve(&mask.rotation_degrees, domain, &owner, values::finite);
            self.curve(&mask.feather_pixels, domain, &owner, values::nonnegative);
            self.curve(&mask.expansion_pixels, domain, &owner, values::finite);
        }
        for stage in &apply.stages {
            let ResolvedApplyOperation::Effect { parameters, .. } = &stage.operation else {
                continue;
            };
            for parameter in parameters.values() {
                if let ParameterValue::NumberCurve { value } = parameter {
                    self.curve(value, domain, &stage.id.to_string(), values::finite);
                }
            }
        }
    }

    fn clip(&mut self, clip: &ResolvedClip) {
        let domain = clip.record_range.duration;
        let owner = clip.id.to_string();
        if let Some(visual) = &clip.visual {
            self.curve(&visual.transform.position, domain, &owner, values::point);
            self.curve(
                &visual.transform.scale,
                domain,
                &owner,
                values::positive_vec,
            );
            self.curve(
                &visual.transform.rotation_degrees,
                domain,
                &owner,
                values::finite,
            );
            if let Some(crop) = &visual.transform.crop {
                self.curve(crop, domain, &owner, values::rect);
            }
            self.curve(&visual.opacity, domain, &owner, values::unit_number);
            for mask in &visual.masks {
                self.curve(&mask.position, domain, &owner, values::unit_vec);
                self.curve(&mask.scale, domain, &owner, values::positive_vec);
                self.curve(&mask.rotation_degrees, domain, &owner, values::finite);
                self.curve(&mask.feather_pixels, domain, &owner, values::nonnegative);
                self.curve(&mask.expansion_pixels, domain, &owner, values::finite);
            }
        }
        if let Some(audio) = &clip.audio {
            self.curve(&audio.gain, domain, &owner, values::nonnegative);
            self.curve(&audio.pan, domain, &owner, values::pan);
        }
        text::validate(self, clip, domain, &owner);
        for effect in &clip.effects {
            let effect_owner = effect.id.to_string();
            if !valid_range(effect.active_range, domain, self.timebase) {
                self.check.push(
                    "PLAN_EFFECT_RANGE_INVALID",
                    Some(effect_owner.clone()),
                    "effect range must be a valid clip-local interval",
                );
            }
            for parameter in effect.parameters.values() {
                if let ParameterValue::NumberCurve { value } = parameter {
                    self.curve(value, domain, &effect_owner, values::finite);
                }
            }
        }
    }

    fn curve<T: values::SpringValue>(
        &mut self,
        curve: &Animatable<T>,
        domain: RationalTime,
        owner: &str,
        valid_value: fn(&T) -> bool,
    ) {
        let Animatable::Keyframes { keyframes } = curve else {
            if let Animatable::Constant { value } = curve {
                if !valid_value(value) {
                    self.invalid(owner);
                }
            }
            return;
        };
        for keyframe in keyframes {
            if !self.ids.insert(keyframe.id.to_string()) {
                self.check.push(
                    "PLAN_KEYFRAME_DUPLICATE",
                    Some(keyframe.id.to_string()),
                    "keyframe IDs must be unique across the render plan",
                );
            }
        }
        if keyframes.is_empty()
            || !keyframes
                .iter()
                .all(|key| self.valid_key(key, domain, valid_value))
            || !keyframes
                .windows(2)
                .all(|pair| pair[0].time.value < pair[1].time.value)
            || !values::spring_ranges_valid(keyframes, valid_value)
        {
            self.invalid(owner);
        }
    }

    fn valid_key<T>(
        &self,
        key: &Keyframe<T>,
        domain: RationalTime,
        valid_value: fn(&T) -> bool,
    ) -> bool {
        veac_plan::canonical::KeyframeId::new(key.id.as_str()).is_ok()
            && valid_point(key.time, domain, self.timebase)
            && valid_value(&key.value)
            && values::interpolation(&key.interpolation)
    }

    fn invalid(&mut self, owner: &str) {
        self.check.push(
            "PLAN_ANIMATION_INVALID",
            Some(owner.to_owned()),
            "animation values and keyframes must satisfy the local timeline contract",
        );
    }
}

pub(super) fn valid_point(value: RationalTime, domain: RationalTime, timebase: u32) -> bool {
    value.is_valid()
        && domain.is_valid()
        && value.value >= 0
        && domain.value > 0
        && value.timescale == timebase
        && domain.timescale == timebase
        && value.value <= domain.value
}

fn valid_range(
    range: veac_plan::canonical::TimeRange,
    domain: RationalTime,
    timebase: u32,
) -> bool {
    range.start.is_valid()
        && range.duration.is_valid()
        && range.start.value >= 0
        && range.duration.value > 0
        && range.start.timescale == timebase
        && range.duration.timescale == timebase
        && domain.is_valid()
        && domain.timescale == timebase
        && range
            .end()
            .is_ok_and(|end| end.value <= domain.value && end.timescale == timebase)
}
