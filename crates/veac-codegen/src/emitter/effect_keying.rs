use super::{effects, effects::EffectSpec, CodegenErrors, EmitContext};
use veac_plan::canonical::{Color, EffectKind, EffectParameter};

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    effect: EffectSpec<'_>,
    input: &str,
    enable: &str,
) -> Result<String, CodegenErrors> {
    match effect.effect.kind() {
        EffectKind::VideoChromaKey => Ok(chroma(context, effect, input, enable)),
        EffectKind::VideoLumaKey => Ok(luma(context, effect, input, enable)),
        EffectKind::VideoChromaSpill => Ok(spill(context, effect, input, enable)),
        _ => unreachable!("keying dispatcher received another effect"),
    }
}

fn chroma(
    context: &mut EmitContext<'_>,
    effect: EffectSpec<'_>,
    input: &str,
    enable: &str,
) -> String {
    let color = color(effect);
    let kind = effect.effect.kind();
    effects::dynamic_filter(
        context,
        effect,
        input,
        enable,
        "colorkey",
        &format!(
            "color=0x{:02X}{:02X}{:02X}",
            color.red, color.green, color.blue
        ),
        &[
            effects::RuntimeNumber::bounded(kind, EffectParameter::Similarity, "similarity", 0.1),
            effects::RuntimeNumber::bounded(kind, EffectParameter::Blend, "blend", 0.0),
        ],
    )
}

fn luma(
    context: &mut EmitContext<'_>,
    effect: EffectSpec<'_>,
    input: &str,
    enable: &str,
) -> String {
    let kind = effect.effect.kind();
    let mut label = effects::dynamic_filter(
        context,
        effect,
        input,
        enable,
        "lumakey",
        "",
        &[
            effects::RuntimeNumber::bounded(kind, EffectParameter::Threshold, "threshold", 0.0),
            effects::RuntimeNumber::bounded(kind, EffectParameter::Tolerance, "tolerance", 0.01),
            effects::RuntimeNumber::bounded(kind, EffectParameter::Softness, "softness", 0.0),
        ],
    );
    if boolean(effect, EffectParameter::Invert) {
        label = context.graph.filter(
            &[&label],
            format!(
                "format=rgba64le,geq=r='r(X\\,Y)':g='g(X\\,Y)':b='b(X\\,Y)':a='65535-alpha(X\\,Y)':{enable}"
            ),
            "effectv",
        );
    }
    label
}

fn spill(
    context: &mut EmitContext<'_>,
    effect: EffectSpec<'_>,
    input: &str,
    enable: &str,
) -> String {
    let screen = if color(effect).blue > color(effect).green {
        "blue"
    } else {
        "green"
    };
    let kind = effect.effect.kind();
    effects::dynamic_filter(
        context,
        effect,
        input,
        enable,
        "despill",
        &format!("type={screen}"),
        &[
            effects::RuntimeNumber::bounded(kind, EffectParameter::Amount, "mix", 0.5),
            effects::RuntimeNumber::bounded(kind, EffectParameter::Range, "expand", 0.0),
        ],
    )
}

fn color(effect: EffectSpec<'_>) -> Color {
    effect
        .effect
        .color(EffectParameter::Color)
        .copied()
        .unwrap_or(Color {
            red: 0,
            green: 255,
            blue: 0,
            alpha: 255,
        })
}

fn boolean(effect: EffectSpec<'_>, parameter: EffectParameter) -> bool {
    effect.effect.boolean(parameter) == Some(true)
}
