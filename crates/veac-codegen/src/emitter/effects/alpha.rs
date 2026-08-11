use super::super::{CodegenErrors, EmitContext};
use super::EffectSpec;
use veac_plan::canonical::{Animatable, EffectKind, EffectParameter};

pub(super) fn apply<F>(
    context: &mut EmitContext<'_>,
    effect: EffectSpec<'_>,
    input: &str,
    window: &str,
    apply_effect: F,
) -> Result<String, CodegenErrors>
where
    F: FnOnce(&mut EmitContext<'_>, String, &str) -> Result<String, CodegenErrors>,
{
    if expands_alpha(effect.effect.kind()) {
        let Some(activity) = expansion_activity(context, effect, window) else {
            return Ok(input.to_owned());
        };
        let enable = format!("enable='{activity}'");
        let input = context.graph.filter(
            &[input],
            format!("premultiply=inplace=1:planes=7:{enable}"),
            "effectpremultiplyv",
        );
        let processed = apply_effect(context, input, &enable)?;
        return Ok(context.graph.filter(
            &[&processed],
            format!("unpremultiply=inplace=1:planes=7:{enable}"),
            "effectunpremultiplyv",
        ));
    }
    let enable = format!("enable='{window}'");
    let (color, source_alpha) = split_input(context, input);
    let processed = apply_effect(context, color, &enable)?;
    if replaces_alpha(effect.effect.kind()) {
        combine_key_alpha(context, &processed, &source_alpha)
    } else {
        Ok(attach(
            context,
            &processed,
            &source_alpha,
            "effectalphamergev",
        ))
    }
}

fn expands_alpha(kind: EffectKind) -> bool {
    kind == EffectKind::VideoDirectionalBlur
}

fn expansion_activity(
    context: &EmitContext<'_>,
    effect: EffectSpec<'_>,
    window: &str,
) -> Option<String> {
    match effect.effect.curve(EffectParameter::Radius) {
        Some(Animatable::Constant { value }) if *value <= 0.0 => None,
        Some(Animatable::Constant { .. }) => Some(window.to_owned()),
        Some(Animatable::Keyframes { keyframes })
            if keyframes.iter().all(|keyframe| keyframe.value <= 0.0) =>
        {
            None
        }
        Some(Animatable::Keyframes { .. } | Animatable::Binding { .. }) => {
            let radius =
                super::number_expression_at(context, effect, EffectParameter::Radius, 0.0, "t");
            Some(format!("({window})*gt(({radius})\\,0)"))
        }
        None => None,
    }
}

fn replaces_alpha(kind: EffectKind) -> bool {
    matches!(kind, EffectKind::VideoChromaKey | EffectKind::VideoLumaKey)
}

fn split_input(context: &mut EmitContext<'_>, input: &str) -> (String, String) {
    let (color, alpha) = context.graph.split(input, "effectalphasplitv");
    let color = super::super::rgb_planes::without_alpha(&mut context.graph, &color, "effectcolorv");
    let alpha = extract(context, &alpha, "effectsourcealphav");
    (color, alpha)
}

fn combine_key_alpha(
    context: &mut EmitContext<'_>,
    processed: &str,
    source_alpha: &str,
) -> Result<String, CodegenErrors> {
    let (color, key_alpha) = context.graph.split(processed, "keyalphasplitv");
    let key_alpha = extract(context, &key_alpha, "keyalphav");
    let alpha = context.graph.filter(
        &[source_alpha, &key_alpha],
        "blend=all_expr='A*B/65535'",
        "keycombinedalphav",
    );
    Ok(attach(context, &color, &alpha, "keyalphamergev"))
}

fn extract(context: &mut EmitContext<'_>, input: &str, prefix: &str) -> String {
    context.graph.filter(
        &[input],
        "format=rgba64le,alphaextract,format=gray16le",
        prefix,
    )
}

fn attach(context: &mut EmitContext<'_>, color: &str, alpha: &str, prefix: &str) -> String {
    let color =
        super::super::rgb_planes::without_alpha(&mut context.graph, color, "effectoutputcolorv");
    super::super::alpha_merge::apply(context, &color, alpha, "gbrap16le", prefix)
}
