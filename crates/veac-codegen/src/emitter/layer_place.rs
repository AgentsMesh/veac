use veac_plan::{EffectiveVisualProperties, ResolvedClip, ResolvedSequence};

use super::{
    generated, geometry, layer, layer_placement, shadow, time, visual_extent, visual_pipeline,
    EmitContext,
};

pub(super) fn place(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
    prepared: visual_pipeline::PreparedLayer,
) -> layer::RenderedLayer {
    let layer = context
        .graph
        .filter(&[&prepared.label], "format=gbrap16le", "layerprecisionv");
    let style = visual.card.as_ref().and_then(|card| card.shadow.as_ref());
    let split = style
        .map(|style| shadow::render(context, &layer, style, prepared.pivot_x, prepared.pivot_y));
    let foreground_layer = split
        .as_ref()
        .map_or_else(|| layer.clone(), |streams| streams.foreground.clone());
    let (x, y) = geometry::canvas_position(
        visual,
        "t",
        prepared.pivot_x,
        prepared.pivot_y,
        context.canvas,
    );
    if x.contains('t') || y.contains('t') {
        return place_animated(context, clip, visual, prepared, &foreground_layer, split);
    }
    let extent = visual_extent::placement_extent(context.plan, sequence, clip, visual)
        .expect("validated visual placement has a bounded extent");
    let foreground = if !visual_extent::requires_canvas_placement(clip, visual) {
        foreground_layer
    } else {
        layer_placement::place_on_canvas(context, &foreground_layer, &x, &y, context.canvas, extent)
    };
    let shadow = split.map(|streams| {
        layer_placement::place_on_canvas(
            context,
            &streams.shadow,
            &shifted(&x, streams.x_delta),
            &shifted(&y, streams.y_delta),
            context.canvas,
            extent,
        )
    });
    layer::RenderedLayer { foreground, shadow }
}

fn place_animated(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
    prepared: visual_pipeline::PreparedLayer,
    layer: &str,
    split: Option<shadow::Streams>,
) -> layer::RenderedLayer {
    let (x, y) = geometry::canvas_overlay_position(
        visual,
        "t",
        prepared.pivot_x,
        prepared.pivot_y,
        context.canvas,
    );
    let duration = time::seconds(clip.record_range.duration);
    let canvas = generated::high_precision_canvas(context, clip, "placementcanvasv");
    let foreground =
        layer_placement::place_animated_canvas(context, &canvas, layer, &x, &y, "0", &duration);
    let shadow = split.map(|streams| {
        let canvas = generated::high_precision_canvas(context, clip, "shadowcanvasv");
        layer_placement::place_animated_canvas(
            context,
            &canvas,
            &streams.shadow,
            &shifted(&x, streams.x_delta),
            &shifted(&y, streams.y_delta),
            "0",
            &duration,
        )
    });
    layer::RenderedLayer { foreground, shadow }
}

fn shifted(position: &str, delta: f64) -> String {
    let delta = time::number(delta);
    if delta.starts_with('-') {
        format!("({position}){delta}")
    } else {
        format!("({position})+{delta}")
    }
}
