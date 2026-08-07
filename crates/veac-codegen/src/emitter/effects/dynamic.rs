use std::fmt::Write as _;

use sha2::{Digest, Sha256};

use super::super::{animation, time, EmitContext};
use super::EffectSpec;
use veac_plan::canonical::{Animatable, EffectParameter};

#[derive(Clone, Copy)]
pub(in crate::emitter) struct RuntimeNumber {
    parameter: EffectParameter,
    option: &'static str,
    default: f64,
    scale: f64,
}

impl RuntimeNumber {
    pub(in crate::emitter) const fn new(
        parameter: EffectParameter,
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
        .filter_map(|parameter| command(context, effect, target, *parameter))
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

fn command(
    context: &EmitContext<'_>,
    effect: EffectSpec<'_>,
    target: &str,
    parameter: RuntimeNumber,
) -> Option<String> {
    let Some(value @ (Animatable::Keyframes { .. } | Animatable::Binding { .. })) =
        effect.effect.curve(parameter.parameter)
    else {
        return None;
    };
    let expression = scaled(
        animation::number(context.plan, effect.owner, value, "T"),
        parameter.scale,
    );
    let escaped = expression.replace('\\', "\\\\").replace(';', "\\\\;");
    Some(format!("[expr] {target} {} {escaped}", parameter.option))
}

fn initial(effect: EffectSpec<'_>, parameter: RuntimeNumber) -> String {
    let value = match effect.effect.curve(parameter.parameter) {
        Some(Animatable::Constant { value }) => *value,
        Some(Animatable::Keyframes { keyframes }) => keyframes
            .first()
            .map_or(parameter.default, |keyframe| keyframe.value),
        Some(Animatable::Binding { .. }) => parameter.default,
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
    // FFmpeg sendcmd silently stops resolving overly long filter instance names.
    let digest = Sha256::digest(effect.id.as_bytes());
    let mut output = String::with_capacity(37);
    output.push_str("veac_");
    for byte in &digest[..16] {
        write!(&mut output, "{byte:02x}").expect("writing to a string cannot fail");
    }
    output
}
