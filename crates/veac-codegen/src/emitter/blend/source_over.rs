use super::{EmitContext, Placement};

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    base: &str,
    source: &str,
    placement: &Placement,
) -> String {
    let source = position(context, source, placement);
    apply_timeline(context, base, &source, placement)
}

pub(super) fn apply_opaque(
    context: &mut EmitContext<'_>,
    base: &str,
    source: &str,
    placement: &Placement,
) -> String {
    // With an opaque base, straight-alpha source-over is a masked color merge; base alpha stays one.
    let source = position(context, source, placement);
    let base = context.graph.filter(&[base], "format=gbrap16le", "soob");
    let source = context.graph.filter(&[&source], "format=gbrap16le", "soos");
    let (source_color, source_alpha) = context.graph.split(&source, "soos");
    let source_alpha = extract_alpha(context, &source_alpha, "sooa");
    context.graph.filter(
        &[&base, &source_color, &source_alpha],
        format!(
            "maskedmerge=planes=7:enable='gte(t,{})*lt(t,{})'",
            placement.start, placement.end
        ),
        "soo",
    )
}

pub(super) fn apply_timeline(
    context: &mut EmitContext<'_>,
    base: &str,
    source: &str,
    placement: &Placement,
) -> String {
    // Labels are serialized into every source-over node, so this hot path uses compact prefixes.
    let base = context.graph.filter(&[base], "format=gbrap16le", "sobp");
    let source = context.graph.filter(&[source], "format=gbrap16le", "sosp");
    let (base_color_raw, base_alpha_raw) = context.graph.split(&base, "sob");
    let (source_color_raw, source_alpha_raw) = context.graph.split(&source, "sos");
    let base_color =
        super::super::rgb_planes::without_alpha(&mut context.graph, &base_color_raw, "sobc");
    let source_color =
        super::super::rgb_planes::without_alpha(&mut context.graph, &source_color_raw, "sosc");
    let base_alpha = extract_alpha(context, &base_alpha_raw, "soba");
    let source_alpha = extract_alpha(context, &source_alpha_raw, "sosa");
    let source_alpha = context.graph.filter(
        &[&source_alpha],
        format!(
            r"geq=lum='if(gte(T\,{})*lt(T\,{})\,lum(X\,Y)\,0)'",
            placement.start, placement.end
        ),
        "sot",
    );
    compose(
        context,
        &base_color,
        &base_alpha,
        &source_color,
        &source_alpha,
    )
}

pub(super) fn position(
    context: &mut EmitContext<'_>,
    source: &str,
    placement: &Placement,
) -> String {
    context.graph.filter(
        &[source],
        format!(
            concat!(
                "setpts=PTS-STARTPTS,",
                "tpad=start_mode=add:start_duration={}:",
                "stop_mode=add:stop=1:color=black@0"
            ),
            placement.start
        ),
        "sop",
    )
}

fn compose(
    context: &mut EmitContext<'_>,
    base_color: &str,
    base_alpha: &str,
    source_color: &str,
    source_alpha: &str,
) -> String {
    let (base_mask, base_output) = context.graph.split(base_alpha, "sobs");
    let (source_mask, source_rest) = context.graph.split(source_alpha, "soss");
    let (source_inverse, source_output) = context.graph.split(&source_rest, "soss");
    let base_premultiplied = premultiply(context, base_color, &base_mask, "sobm");
    let source_premultiplied = premultiply(context, source_color, &source_mask, "sosm");
    let inverse = context.graph.filter(&[&source_inverse], "negate", "soi");
    let remaining_base = premultiply(context, &base_premultiplied, &inverse, "sor");
    let color = context.graph.filter(
        &[&remaining_base, &source_premultiplied],
        "blend=all_expr='min(65535,A+B)'",
        "soc",
    );
    let alpha = context.graph.filter(
        &[&base_output, &source_output],
        "blend=all_expr='B+A*(65535-B)/65535'",
        "soa",
    );
    let (unpremultiply_alpha, output_alpha) = context.graph.split(&alpha, "soas");
    let color = context.graph.filter(
        &[&color, &unpremultiply_alpha],
        "unpremultiply=planes=7",
        "sost",
    );
    super::super::alpha_merge::apply(context, &color, &output_alpha, "gbrap16le", "som")
}

fn extract_alpha(context: &mut EmitContext<'_>, input: &str, prefix: &str) -> String {
    context.graph.filter(
        &[input],
        "format=rgba64le,alphaextract,format=gray16le",
        prefix,
    )
}

fn premultiply(context: &mut EmitContext<'_>, color: &str, alpha: &str, prefix: &str) -> String {
    context
        .graph
        .filter(&[color, alpha], "premultiply=planes=7", prefix)
}
