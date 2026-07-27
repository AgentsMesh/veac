use std::fmt::Write as _;

use super::super::{animation, time, EmitContext};
use super::EffectSpec;
use veac_plan::canonical::{Animatable, ParameterValue};

#[derive(Clone, Copy)]
pub(in crate::emitter) struct RuntimeNumber {
    parameter: &'static str,
    option: &'static str,
    default: f64,
    scale: f64,
}

impl RuntimeNumber {
    pub(in crate::emitter) const fn new(
        parameter: &'static str,
        option: &'static str,
        default: f64,
        scale: f64,
    ) -> Self {
        Self {
            parameter,
            option,
            default,
            scale,
        }
    }
}

pub(in crate::emitter) fn filter(
    context: &mut EmitContext<'_>,
    effect: EffectSpec<'_>,
    input: &str,
    enable: &str,
    filter: &str,
    fixed: &str,
    parameters: &[RuntimeNumber],
) -> String {
    let target = format!("{filter}@{}", instance(effect));
    let mut options = Vec::with_capacity(parameters.len() + 2);
    if !fixed.is_empty() {
        options.push(fixed.to_owned());
    }
    options.extend(
        parameters
            .iter()
            .map(|parameter| format!("{}={}", parameter.option, initial(effect, *parameter))),
    );
    options.push(enable.to_owned());
    let input = command_node(context, effect, input, &target, parameters);
    context.graph.filter(
        &[&input],
        format!("{target}={}", options.join(":")),
        "effectv",
    )
}

fn command_node(
    context: &mut EmitContext<'_>,
    effect: EffectSpec<'_>,
    input: &str,
    target: &str,
    parameters: &[RuntimeNumber],
) -> String {
    let commands: Vec<_> = parameters
        .iter()
        .filter_map(|parameter| command(effect, target, *parameter))
        .collect();
    if commands.is_empty() {
        return input.to_owned();
    }
    let start = time::seconds(effect.active_range.start);
    let end = time::end(effect.active_range);
    context.graph.filter(
        &[input],
        format!("sendcmd=c='{start}-{end} {}'", commands.join(",")),
        "effectcmd",
    )
}

fn command(effect: EffectSpec<'_>, target: &str, parameter: RuntimeNumber) -> Option<String> {
    let Some(ParameterValue::NumberCurve {
        value: value @ Animatable::Keyframes { .. },
    }) = effect.parameters.get(parameter.parameter)
    else {
        return None;
    };
    let expression = scaled(animation::number(value, "T"), parameter.scale);
    let escaped = expression.replace('\\', "\\\\").replace(';', "\\\\;");
    Some(format!("[expr] {target} {} {escaped}", parameter.option))
}

fn initial(effect: EffectSpec<'_>, parameter: RuntimeNumber) -> String {
    let value = match effect.parameters.get(parameter.parameter) {
        Some(ParameterValue::Number { value }) => *value,
        Some(ParameterValue::NumberCurve {
            value: Animatable::Constant { value },
        }) => *value,
        Some(ParameterValue::NumberCurve {
            value: Animatable::Keyframes { keyframes },
        }) => keyframes
            .first()
            .map_or(parameter.default, |keyframe| keyframe.value),
        _ => parameter.default,
    };
    time::number(value * parameter.scale)
}

fn scaled(expression: String, scale: f64) -> String {
    if scale == 1.0 {
        expression
    } else {
        format!("({expression})*{}", time::number(scale))
    }
}

fn instance(effect: EffectSpec<'_>) -> String {
    let mut output = String::from("veac_");
    for byte in effect.id.bytes() {
        write!(&mut output, "{byte:02x}").expect("writing to a string cannot fail");
    }
    output
}
