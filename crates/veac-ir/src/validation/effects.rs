use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn effect(
        &mut self,
        effect: &EffectInstance,
        duration: RationalTime,
        timebase: u32,
        path: &str,
        item_id: &str,
    ) {
        let effect_path = format!("{path}/effects/{}", effect.id);
        self.check_id(effect.id.is_valid(), effect.id.as_str(), &effect_path);
        if !self.effect_ids.insert(effect.id.to_string()) {
            self.duplicate("DUPLICATE_EFFECT_ID", effect.id.as_str(), &effect_path);
        }
        let specification = built_in_effect(&effect.effect_type);
        if specification.is_none() {
            self.value_error("UNKNOWN_EFFECT", &effect_path, item_id);
        }
        if let Some(range) = effect.enable_range {
            self.time_range(
                range,
                timebase,
                "EFFECT_RANGE",
                &format!("{effect_path}/enable_range"),
                item_id,
            );
            if range.end().is_ok_and(|end| end > duration) {
                self.value_error("EFFECT_RANGE", &effect_path, item_id);
            }
        }
        let context = EffectContext {
            duration,
            timebase,
            path: &effect_path,
            item_id,
        };
        for (name, value) in &effect.parameters {
            self.effect_parameter(specification, name, value, context);
        }
    }

    fn effect_parameter(
        &mut self,
        specification: Option<EffectSpec>,
        name: &str,
        value: &ParameterValue,
        context: EffectContext<'_>,
    ) {
        if let Some(specification) = specification {
            match specification
                .parameters
                .iter()
                .find(|parameter| parameter.name == name)
            {
                None => self.value_error("UNKNOWN_EFFECT_PARAMETER", context.path, context.item_id),
                Some(parameter) if !parameter_matches(*parameter, value) => {
                    self.value_error("EFFECT_PARAMETER_TYPE", context.path, context.item_id);
                }
                Some(_) => {}
            }
        }
        match value {
            ParameterValue::Number { value } if !value.is_finite() => {
                self.value_error("EFFECT_PARAMETER", context.path, context.item_id)
            }
            ParameterValue::Integer { value } if !crate::time::safe_i64(*value) => {
                self.value_error("EFFECT_PARAMETER", context.path, context.item_id)
            }
            ParameterValue::Vec2 { value } => self.finite_vec(
                *value,
                false,
                "EFFECT_PARAMETER",
                context.path,
                context.item_id,
            ),
            ParameterValue::Time { value } => self.time(
                *value,
                context.timebase,
                false,
                "EFFECT_PARAMETER",
                context.path,
                context.item_id,
            ),
            ParameterValue::NumberCurve { value } => self.animatable(
                value,
                context.duration,
                context.timebase,
                context.path,
                context.item_id,
                |number| number.is_finite(),
            ),
            _ => {}
        }
    }
}

#[derive(Clone, Copy)]
struct EffectContext<'a> {
    duration: RationalTime,
    timebase: u32,
    path: &'a str,
    item_id: &'a str,
}
