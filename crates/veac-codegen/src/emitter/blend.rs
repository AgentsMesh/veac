use veac_plan::canonical::BlendMode;

use super::EmitContext;

mod source_over;

pub(super) struct Placement {
    pub start: String,
    pub end: String,
    pub mode: BlendMode,
}

pub(super) fn composite(
    context: &mut EmitContext<'_>,
    base: String,
    layer: &str,
    placement: Placement,
) -> String {
    composite_with_base(context, base, layer, placement, false)
}

pub(super) fn composite_with_base(
    context: &mut EmitContext<'_>,
    base: String,
    layer: &str,
    placement: Placement,
    base_is_opaque: bool,
) -> String {
    if placement.mode == BlendMode::Normal {
        if base_is_opaque {
            return source_over::apply_opaque(context, &base, layer, &placement);
        }
        return source_over::apply(context, &base, layer, &placement);
    }
    blend_layer(context, &base, layer, &placement)
}

fn blend_layer(
    context: &mut EmitContext<'_>,
    base: &str,
    layer: &str,
    placement: &Placement,
) -> String {
    let base_main = base.to_owned();
    let full = source_over::position(context, layer, placement);
    let (base_output, base_work) = context.graph.split(&base_main, "blendbasesplit");
    let (base_color_raw, base_alpha_raw) = context.graph.split(&base_work, "blendbaseworksplit");
    let (source_color_raw, source_alpha_raw) = context.graph.split(&full, "layersplit");
    let (source_original_raw, source_blend_raw) =
        context.graph.split(&source_color_raw, "blendcolorsplit");
    let base_color =
        super::rgb_planes::without_alpha(&mut context.graph, &base_color_raw, "blendbasev");
    let source_original =
        super::rgb_planes::without_alpha(&mut context.graph, &source_original_raw, "blendsourcev");
    let source_blend =
        super::rgb_planes::without_alpha(&mut context.graph, &source_blend_raw, "blendcolorv");
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
