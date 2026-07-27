use super::super::{CodegenErrors, EmitContext};
use super::EffectSpec;

pub(super) fn apply<F>(
    context: &mut EmitContext<'_>,
    effect: EffectSpec<'_>,
    input: &str,
    apply_effect: F,
) -> Result<String, CodegenErrors>
where
    F: FnOnce(&mut EmitContext<'_>, String) -> Result<String, CodegenErrors>,
{
    let (color, source_alpha) = split_input(context, input);
    let processed = apply_effect(context, color)?;
    if matches!(effect.effect_type, "video.chroma_key" | "video.luma_key") {
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

fn split_input(context: &mut EmitContext<'_>, input: &str) -> (String, String) {
    let (color, alpha) = context.graph.split(input, "effectalphasplitv");
    let color = context
        .graph
        .filter(&[&color], "format=gbrp16le", "effectcolorv");
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
    let color = context
        .graph
        .filter(&[color], "format=gbrp16le", "effectoutputcolorv");
    super::super::alpha_merge::apply(context, &color, alpha, "gbrap16le", prefix)
}
