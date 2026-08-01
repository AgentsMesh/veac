use std::cmp::Ordering;
use std::collections::BTreeSet;

use veac_plan::canonical::{BlendMode, TrackKind};
use veac_plan::{EffectiveVisualProperties, ResolvedClip, ResolvedSequence, ResolvedTrack};

use super::{apply_stack::ApplyStack, blend, layer, time, transition, CodegenErrors, EmitContext};

pub(super) fn compose(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    mut base: String,
) -> Result<String, CodegenErrors> {
    let mut layers = Vec::new();
    for track in &sequence.tracks {
        if !track.state.visual_enabled
            || (track.kind == TrackKind::Caption && !context.captions_visible())
            || !matches!(
                track.kind,
                TrackKind::Video | TrackKind::Visual | TrackKind::Caption
            )
        {
            continue;
        }
        for clip in &track.clips {
            if let Some(visual) = &clip.visual {
                layers.push((track.order, track.source_order, track, clip, visual));
            }
        }
    }
    layers.sort_by(layer_order);
    let mut matte_sources: BTreeSet<_> = layers
        .iter()
        .filter_map(|layer| {
            layer
                .4
                .track_matte
                .as_ref()
                .map(|matte| matte.source_clip_id.clone())
        })
        .collect();
    matte_sources.extend(sequence.applies.iter().filter_map(|apply| {
        apply
            .matte
            .as_ref()
            .map(|matte| matte.source_clip_id.clone())
    }));
    let mut applies = ApplyStack::new(context, sequence);
    for (position, track) in sequence.tracks.iter().enumerate() {
        base = applies.checkpoint(context, position, track, base);
        for (_, _, _, clip, visual) in layers.iter().filter(|layer| layer.2.id == track.id) {
            if matte_sources.contains(&clip.id) {
                continue;
            }
            base = compose_layer(context, sequence, base, track, clip, visual)?;
            for target in applies.targets(position) {
                applies.add_layer(context, &target, sequence, track, clip, visual)?;
            }
        }
        for value in applies.take_due(position) {
            base = applies.apply(context, sequence, base, value)?;
        }
    }
    Ok(base)
}

pub(super) fn compose_layer(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    mut base: String,
    track: &ResolvedTrack,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
) -> Result<String, CodegenErrors> {
    base = overlay(context, sequence, &base, clip, visual)?;
    for effect in track
        .transitions
        .iter()
        .filter(|effect| effect.incoming_clip_id == clip.id)
    {
        base = transition::compose(context, sequence, base, track, effect)?;
    }
    Ok(base)
}

fn overlay(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    base: &str,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
) -> Result<String, CodegenErrors> {
    let rendered = layer::render_with_shadow(context, sequence, clip, visual)?;
    let layer = context.graph.filter(
        &[&rendered.foreground],
        format!("setpts=PTS+{}/TB", time::seconds(clip.record_range.start)),
        "offsetv",
    );
    let start = time::seconds(clip.record_range.start);
    let end = time::end(clip.record_range);
    let base = match rendered.shadow {
        Some(shadow) => {
            let shadow = context.graph.filter(
                &[&shadow],
                format!("setpts=PTS+{start}/TB"),
                "shadowoffsetv",
            );
            blend::composite(
                context,
                base.to_owned(),
                &shadow,
                blend::Placement {
                    start: start.clone(),
                    end: end.clone(),
                    mode: BlendMode::Normal,
                },
            )
        }
        None => base.to_owned(),
    };
    Ok(blend::composite(
        context,
        base,
        &layer,
        blend::Placement {
            start,
            end,
            mode: visual.compositing.blend_mode,
        },
    ))
}

fn layer_order(
    left: &(
        i32,
        u32,
        &ResolvedTrack,
        &ResolvedClip,
        &EffectiveVisualProperties,
    ),
    right: &(
        i32,
        u32,
        &ResolvedTrack,
        &ResolvedClip,
        &EffectiveVisualProperties,
    ),
) -> Ordering {
    left.0
        .cmp(&right.0)
        .then(left.4.compositing.z_index.cmp(&right.4.compositing.z_index))
        .then(left.1.cmp(&right.1))
        .then_with(|| {
            left.3
                .record_range
                .start
                .partial_cmp(&right.3.record_range.start)
                .unwrap_or(Ordering::Equal)
        })
        .then(left.3.source_order.cmp(&right.3.source_order))
        .then(left.3.id.cmp(&right.3.id))
}
