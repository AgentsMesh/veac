use veac_plan::canonical::{BlendMode, Shadow};

use super::{time, EmitContext};

mod source_over;

pub(super) struct Placement<'a> {
    pub x: &'a str,
    pub y: &'a str,
    pub start: String,
    pub end: String,
    pub mode: BlendMode,
    pub shadow: Option<&'a Shadow>,
}

pub(super) fn composite(
    context: &mut EmitContext<'_>,
    mut base: String,
    layer: &str,
    placement: Placement<'_>,
) -> String {
    let mut layer = layer.to_owned();
    if let Some(shadow) = placement.shadow {
        let (main, silhouette) = context.graph.split(&layer, "cardsplit");
        layer = main;
        base = shadow_layer(context, base, &silhouette, &placement, shadow);
    }
    if placement.mode == BlendMode::Normal {
        if placement.x == "0" && placement.y == "0" {
            return source_over::apply(context, &base, &layer, &placement);
        }
        return overlay(context, &base, &layer, &placement);
    }
    blend_layer(context, &base, &layer, &placement)
}

fn shadow_layer(
    context: &mut EmitContext<'_>,
    base: String,
    silhouette: &str,
    placement: &Placement<'_>,
    shadow: &Shadow,
) -> String {
    let pad = (shadow.blur_pixels * 2.0).ceil();
    let color = shadow.color;
    let label = context.graph.filter(
        &[silhouette],
        format!(
            "pad=iw+{p2}:ih+{p2}:{pad}:{pad}:color=black@0,format=rgba,geq=r={}:g={}:b={}:a='alpha(X\\,Y)*{}',gblur=sigma={}:planes=8",
            color.red,
            color.green,
            color.blue,
            time::number(shadow.opacity),
            time::number(shadow.blur_pixels),
            p2 = pad * 2.0
        ),
        "shadowv",
    );
    let x = format!("({})-{pad}+{}", placement.x, time::number(shadow.offset.x));
    let y = format!("({})-{pad}+{}", placement.y, time::number(shadow.offset.y));
    overlay(
        context,
        &base,
        &label,
        &Placement {
            x: &x,
            y: &y,
            start: placement.start.clone(),
            end: placement.end.clone(),
            mode: BlendMode::Normal,
            shadow: None,
        },
    )
}

fn blend_layer(
    context: &mut EmitContext<'_>,
    base: &str,
    layer: &str,
    placement: &Placement<'_>,
) -> String {
    let (base_main, canvas_seed) = context.graph.split(base, "basesplit");
    let canvas = context.graph.filter(
        &[&canvas_seed],
        "format=rgba,colorchannelmixer=rr=0:gg=0:bb=0:aa=0",
        "canvasv",
    );
    let full = overlay(context, &canvas, layer, placement);
    let (base_output, base_work) = context.graph.split(&base_main, "blendbasesplit");
    let (base_color_raw, base_alpha_raw) = context.graph.split(&base_work, "blendbaseworksplit");
    let (source_color_raw, source_alpha_raw) = context.graph.split(&full, "layersplit");
    let (source_original_raw, source_blend_raw) =
        context.graph.split(&source_color_raw, "blendcolorsplit");
    let base_color = context
        .graph
        .filter(&[&base_color_raw], "format=gbrp16le", "blendbasev");
    let source_original =
        context
            .graph
            .filter(&[&source_original_raw], "format=gbrp16le", "blendsourcev");
    let source_blend = context
        .graph
        .filter(&[&source_blend_raw], "format=gbrp16le", "blendcolorv");
    let blended = context.graph.filter(
        &[&base_color, &source_blend],
        format!("blend=all_mode={}", mode_name(placement.mode)),
        "blendv",
    );
    let base_alpha = context.graph.filter(
        &[&base_alpha_raw],
        "format=rgba64le,alphaextract,format=gray16le",
        "blendbasealphav",
    );
    let color = context.graph.filter(
        &[&source_original, &blended, &base_alpha],
        "maskedmerge=planes=7",
        "blendstraightv",
    );
    let source_alpha = context.graph.filter(
        &[&source_alpha_raw],
        "format=rgba64le,alphaextract,format=gray16le",
        "blendalpha",
    );
    let source =
        super::alpha_merge::apply(context, &color, &source_alpha, "gbrap16le", "blendmergev");
    source_over::apply_timeline(context, &base_output, &source, placement)
}

fn overlay(
    context: &mut EmitContext<'_>,
    base: &str,
    layer: &str,
    placement: &Placement<'_>,
) -> String {
    context.graph.filter(
        &[base, layer],
        format!(
            "overlay=x='{}':y='{}':eval=frame:format=auto:alpha=straight:enable='gte(t,{})*lt(t,{})'",
            placement.x, placement.y, placement.start, placement.end
        ),
        "overlayv",
    )
}

fn mode_name(mode: BlendMode) -> &'static str {
    match mode {
        BlendMode::Normal => "normal",
        BlendMode::Multiply => "multiply",
        BlendMode::Screen => "screen",
        BlendMode::Overlay => "overlay",
        BlendMode::Darken => "darken",
        BlendMode::Lighten => "lighten",
        BlendMode::ColorDodge => "dodge",
        BlendMode::ColorBurn => "burn",
        BlendMode::HardLight => "hardlight",
        BlendMode::SoftLight => "softlight",
        BlendMode::Difference => "difference",
        BlendMode::Exclusion => "exclusion",
    }
}
