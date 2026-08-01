use std::collections::BTreeMap;

use crate::authoring::{EffectModifierDecl, EffectParameterValue, ParameterDecl};
use veac_ir::{Color, EffectInstance, ParameterValue};

use super::animation;
use super::context::Context;
use super::ids;
use super::value::range;

pub(super) fn lower(context: &mut Context, value: &EffectModifierDecl) -> Option<EffectInstance> {
    let mut parameters = BTreeMap::new();
    for parameter in &value.parameters {
        let lowered = match &parameter.value {
            EffectParameterValue::Number(ParameterDecl::Constant(value)) => {
                ParameterValue::Number {
                    value: super::value::unitless(context, value)?,
                }
            }
            EffectParameterValue::Number(value) => ParameterValue::NumberCurve {
                value: animation::parameter(context, value, super::value::unitless)?,
            },
            EffectParameterValue::Boolean(value) => ParameterValue::Boolean { value: value.value },
            EffectParameterValue::Color(value) => ParameterValue::Color {
                value: color(context, &value.value, value.span)?,
            },
        };
        parameters.insert(parameter.name.value.clone(), lowered);
    }
    Some(EffectInstance {
        id: ids::effect(context, &value.id)?,
        effect_type: value.effect_type.value.clone(),
        enabled: value.enabled.as_ref().is_none_or(|value| value.value),
        enable_range: value
            .record
            .as_ref()
            .and_then(|value| range(context, &value.at, &value.duration)),
        parameters,
    })
}

fn color(context: &mut Context, raw: &str, span: crate::authoring::Span) -> Option<Color> {
    let Some(digits) = raw.strip_prefix('#') else {
        context.error("AUTHORING_LOWER_COLOR", "color must start with #", span);
        return None;
    };
    let (red, green, blue, alpha) = match digits.len() {
        6 => (
            component(&digits[0..2])?,
            component(&digits[2..4])?,
            component(&digits[4..6])?,
            u8::MAX,
        ),
        8 => (
            component(&digits[0..2])?,
            component(&digits[2..4])?,
            component(&digits[4..6])?,
            component(&digits[6..8])?,
        ),
        _ => {
            context.error(
                "AUTHORING_LOWER_COLOR",
                "color must use #rrggbb or #rrggbbaa",
                span,
            );
            return None;
        }
    };
    Some(Color {
        red,
        green,
        blue,
        alpha,
    })
}

fn component(value: &str) -> Option<u8> {
    u8::from_str_radix(value, 16).ok()
}
