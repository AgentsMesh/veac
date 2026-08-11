use super::{
    effects, effects::EffectSpec, process_owner::ProcessOwner, CodegenErrors, EmitContext,
};
use veac_plan::canonical::{EffectKind, EffectParameter};

mod stabilize;

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    owner: ProcessOwner<'_>,
    effect: EffectSpec<'_>,
    input: String,
    enable: &str,
) -> Result<String, CodegenErrors> {
    let kind = effect.effect.kind();
    match kind {
        EffectKind::VideoColorAdjust => Ok(color_adjust(context, effect, &input, enable)),
        EffectKind::VideoBlur => Ok(effects::dynamic_filter(
            context,
            effect,
            &input,
            enable,
            "gblur",
            "",
            &[effects::RuntimeNumber::bounded(
                kind,
                EffectParameter::Radius,
                "sigma",
                0.0,
            )],
        )),
        EffectKind::VideoDirectionalBlur => Ok(effects::dynamic_filter(
            context,
            effect,
            &input,
            enable,
            "dblur",
            "planes=15",
            &[
                effects::RuntimeNumber::bounded(kind, EffectParameter::AngleDegrees, "angle", 0.0),
                effects::RuntimeNumber::bounded(kind, EffectParameter::Radius, "radius", 0.0),
            ],
        )),
        EffectKind::VideoSharpen => Ok(effects::dynamic_filter(
            context,
            effect,
            &input,
            enable,
            "cas",
            "",
            &[effects::RuntimeNumber::scaled(
                kind,
                EffectParameter::Amount,
                "strength",
                0.0,
                0.1,
            )],
        )),
        EffectKind::VideoVignette => Ok(vignette(context, effect, &input, enable)),
        EffectKind::VideoGrain => Ok(grain(context, effect, &input, enable)),
        EffectKind::VideoChromaKey | EffectKind::VideoLumaKey | EffectKind::VideoChromaSpill => {
            super::effect_keying::apply(context, effect, &input, enable)
        }
        EffectKind::VideoStabilize => stabilize::apply(context, owner, effect, &input),
        EffectKind::VideoPluginReferenceMonochromeV1 => {
            Ok(plugin_monochrome(context, effect, &input, enable))
        }
        EffectKind::AudioNormalize => Ok(input),
    }
}

fn plugin_monochrome(
    context: &mut EmitContext<'_>,
    effect: EffectSpec<'_>,
    input: &str,
    enable: &str,
) -> String {
    let amount = effects::number_expression(context, effect, EffectParameter::Amount, 0.0);
    context.graph.filter(
        &[input],
        format!("hue=s='1-({amount})':{enable}"),
        "effectv",
    )
}

fn color_adjust(
    context: &mut EmitContext<'_>,
    effect: EffectSpec<'_>,
    input: &str,
    enable: &str,
) -> String {
    let brightness = effects::number_expression(context, effect, EffectParameter::Brightness, 0.0);
    let contrast = effects::number_expression(context, effect, EffectParameter::Contrast, 1.0);
    let saturation = effects::number_expression(context, effect, EffectParameter::Saturation, 1.0);
    context.graph.filter(
        &[input],
        format!(
            "eq=brightness='{brightness}':contrast='{contrast}':saturation='{saturation}':eval=frame:{enable}"
        ),
        "effectv",
    )
}

fn vignette(
    context: &mut EmitContext<'_>,
    effect: EffectSpec<'_>,
    input: &str,
    enable: &str,
) -> String {
    let amount = effects::number_expression(context, effect, EffectParameter::Amount, 0.0);
    context.graph.filter(
        &[input],
        format!("vignette=angle='PI/(5-4*({amount}))':eval=frame:{enable}"),
        "effectv",
    )
}

fn grain(
    context: &mut EmitContext<'_>,
    effect: EffectSpec<'_>,
    input: &str,
    enable: &str,
) -> String {
    if !effects::has_keyframes(effect, EffectParameter::Amount) {
        let amount = effects::number_expression(context, effect, EffectParameter::Amount, 0.0);
        return context.graph.filter(
            &[input],
            format!("noise=alls='{amount}*100':allf=t+u:{enable}"),
            "effectv",
        );
    }
    let (clean, noisy) = context.graph.split(input, "grain");
    let noisy = context
        .graph
        .filter(&[&noisy], "noise=alls=100:allf=t+u", "grain");
    let amount = effects::number_expression_at(context, effect, EffectParameter::Amount, 0.0, "T");
    context.graph.filter(
        &[&clean, &noisy],
        format!("blend=all_expr='A*(1-({amount}))+B*({amount})':{enable}"),
        "effectv",
    )
}
