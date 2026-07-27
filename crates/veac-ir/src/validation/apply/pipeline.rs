use crate::*;

use super::super::Validator;

impl Validator {
    pub(super) fn apply_pipeline(&mut self, apply: &Apply, timebase: u32, path: &str) {
        if apply.stages.is_empty() || apply.stages.len() > 64 {
            self.value_error("APPLY_STAGE_COUNT", path, apply.id.as_str());
        }
        for stage in &apply.stages {
            let stage_path = format!("{path}/stages/{}", stage.id);
            self.check_id(stage.id.is_valid(), stage.id.as_str(), &stage_path);
            if !self.apply_stage_ids.insert(stage.id.to_string()) {
                self.duplicate("DUPLICATE_APPLY_STAGE_ID", stage.id.as_str(), &stage_path);
            }
            if let Some(range) = stage.active_range {
                self.time_range(
                    range,
                    timebase,
                    "APPLY_STAGE_RANGE",
                    &format!("{stage_path}/active_range"),
                    stage.id.as_str(),
                );
                if range
                    .end()
                    .map_or(true, |end| end > apply.record_range.duration)
                {
                    self.value_error("APPLY_STAGE_RANGE", &stage_path, stage.id.as_str());
                }
            }
            match &stage.operation {
                ApplyOperation::Color { pipeline } => {
                    self.color_pipeline(pipeline, &stage_path, stage.id.as_str());
                }
                ApplyOperation::Effect { effect } => {
                    if effect.enable_range.is_some() {
                        self.value_error("APPLY_EFFECT_RANGE", &stage_path, effect.id.as_str());
                    }
                    self.effect(
                        effect,
                        apply.record_range.duration,
                        timebase,
                        &stage_path,
                        apply.id.as_str(),
                    );
                }
            }
        }
        let mix_path = format!("{path}/mix");
        if apply.mix.masks.len() > 64 {
            self.value_error("APPLY_MASK_COUNT", &mix_path, apply.id.as_str());
        }
        self.animatable(
            &apply.mix.opacity,
            apply.record_range.duration,
            timebase,
            &format!("{mix_path}/opacity"),
            apply.id.as_str(),
            |value| value.is_finite() && (0.0..=1.0).contains(value),
        );
        for (index, mask) in apply.mix.masks.iter().enumerate() {
            self.mask(
                mask,
                apply.record_range.duration,
                timebase,
                &format!("{mix_path}/masks/{index}"),
                apply.id.as_str(),
            );
        }
    }
}
