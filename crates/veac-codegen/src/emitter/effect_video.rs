use super::{
    effects, effects::EffectSpec, process_owner::ProcessOwner, CodegenErrors, EmitContext,
};

mod stabilize;

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    owner: ProcessOwner<'_>,
    effect: EffectSpec<'_>,
    input: String,
    enable: &str,
) -> Result<String, CodegenErrors> {
    match effect.effect_type {
        "video.color_adjust" => Ok(color_adjust(context, effect, &input, enable)),
        "video.blur" => Ok(effects::dynamic_filter(
            context,
            effect,
            &input,
            enable,
            "gblur",
            "",
            &[effects::RuntimeNumber::new("radius", "sigma", 0.0, 1.0)],
        )),
        "video.sharpen" => Ok(effects::dynamic_filter(
            context,
            effect,
            &input,
            enable,
            "cas",
            "",
            &[effects::RuntimeNumber::new("amount", "strength", 0.0, 0.1)],
        )),
        "video.vignette" => Ok(vignette(context, effect, &input, enable)),
        "video.grain" => Ok(grain(context, effect, &input, enable)),
        "video.chroma_key" | "video.luma_key" | "video.chroma_spill" => {
            super::effect_keying::apply(context, effect, &input, enable)
        }
        "video.stabilize" => stabilize::apply(context, owner, effect, &input),
        "audio.normalize" => Ok(input),
        other => Err(effects::unsupported(owner, effect, other)),
    }
}

fn color_adjust(
    context: &mut EmitContext<'_>,
    effect: EffectSpec<'_>,
    input: &str,
    enable: &str,
) -> String {
    let brightness = effects::number_expression(effect, "brightness", 0.0);
    let contrast = effects::number_expression(effect, "contrast", 1.0);
    let saturation = effects::number_expression(effect, "saturation", 1.0);
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
    let amount = effects::number_expression(effect, "amount", 0.0);
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
    if !effects::has_keyframes(effect, "amount") {
        let amount = effects::number_expression(effect, "amount", 0.0);
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
    let amount = effects::number_expression_at(effect, "amount", 0.0, "T");
    context.graph.filter(
        &[&clean, &noisy],
        format!("blend=all_expr='A*(1-({amount}))+B*({amount})':{enable}"),
        "effectv",
    )
}
