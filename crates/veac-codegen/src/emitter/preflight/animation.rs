mod owners;
mod range;
mod text;
mod values;

use std::collections::BTreeSet;

use veac_plan::canonical::{Animatable, Keyframe, RationalTime};
use veac_plan::ResolvedRenderPlan;

use crate::emitter::process_owner::ProcessOwner;

use super::Check;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    let mut curves = Curves {
        check,
        plan,
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
    plan: &'a ResolvedRenderPlan,
    ids: BTreeSet<String>,
    timebase: u32,
}

impl Curves<'_> {
    fn curve<T: values::CurveValue>(
        &mut self,
        curve: &Animatable<T>,
        domain: RationalTime,
        owner: &str,
        process: ProcessOwner<'_>,
        valid_value: fn(&T) -> bool,
    ) {
        if let Animatable::Binding { binding_id } = curve {
            if let Err(error) =
                crate::emitter::temporal::compile_binding(self.plan, binding_id, process, "t")
            {
                self.check.temporal_backend(error);
            }
            return;
        }
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
            || !values::interpolation_ranges_valid(keyframes, valid_value)
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
            && range::point(key.time, domain, self.timebase)
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
