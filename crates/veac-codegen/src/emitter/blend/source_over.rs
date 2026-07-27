use super::{EmitContext, Placement};

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    base: &str,
    source: &str,
    placement: &Placement<'_>,
) -> String {
    let source = timeline_source(context, source, placement);
    apply_timeline(context, base, &source, placement)
}

pub(super) fn apply_timeline(
    context: &mut EmitContext<'_>,
    base: &str,
    source: &str,
    placement: &Placement<'_>,
) -> String {
    let (base_color_raw, base_alpha_raw) = context.graph.split(base, "overbasesplitv");
    let (source_color_raw, source_alpha_raw) = context.graph.split(source, "oversourcesplitv");
    let base_color = format_color(context, &base_color_raw, "overbasecolorv");
    let source_color = format_color(context, &source_color_raw, "oversourcecolorv");
    let base_alpha = extract_alpha(context, &base_alpha_raw, "overbasealphav");
    let source_alpha = extract_alpha(context, &source_alpha_raw, "oversourcealphav");
    let source_alpha = context.graph.filter(
        &[&source_alpha],
        format!(
            r"geq=lum='if(gte(T\,{})*lt(T\,{})\,lum(X\,Y)\,0)'",
            placement.start, placement.end
        ),
        "overtimealphav",
    );
    compose(
        context,
        &base_color,
        &base_alpha,
        &source_color,
        &source_alpha,
    )
}

fn timeline_source(
    context: &mut EmitContext<'_>,
    source: &str,
    placement: &Placement<'_>,
) -> String {
    context.graph.filter(
        &[source],
        format!(
            "setpts=PTS-STARTPTS,tpad=start_mode=add:start_duration={}:stop_mode=clone:stop_duration={}:color=black@0,format=gbrap16le",
            placement.start, placement.end
        ),
        "overtimelinev",
    )
}

fn compose(
    context: &mut EmitContext<'_>,
    base_color: &str,
    base_alpha: &str,
    source_color: &str,
    source_alpha: &str,
) -> String {
    let (base_mask, base_output) = context.graph.split(base_alpha, "overbasealphasplitv");
    let (source_mask, source_rest) = context.graph.split(source_alpha, "oversourcealphasplitv");
    let (source_inverse, source_output) =
        context.graph.split(&source_rest, "oversourcealphasplitv");
    let base_premultiplied = premultiply(context, base_color, &base_mask, "overbasepremulv");
    let source_premultiplied =
        premultiply(context, source_color, &source_mask, "oversourcepremulv");
    let inverse = context
        .graph
        .filter(&[&source_inverse], "negate", "overinversealphav");
    let remaining_base = premultiply(context, &base_premultiplied, &inverse, "overremainingbasev");
    let color = context.graph.filter(
        &[&remaining_base, &source_premultiplied],
        "blend=all_expr='min(65535,A+B)'",
        "overpremulcolorv",
    );
    let alpha = context.graph.filter(
        &[&base_output, &source_output],
        "blend=all_expr='B+A*(65535-B)/65535'",
        "overoutputalphav",
    );
    let (unpremultiply_alpha, output_alpha) = context.graph.split(&alpha, "overalphasplitv");
    let color = context.graph.filter(
        &[&color, &unpremultiply_alpha],
        "unpremultiply=planes=7",
        "overstraightcolorv",
    );
    super::super::alpha_merge::apply(context, &color, &output_alpha, "gbrap16le", "overmergev")
}

fn format_color(context: &mut EmitContext<'_>, input: &str, prefix: &str) -> String {
    context.graph.filter(&[input], "format=gbrp16le", prefix)
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
