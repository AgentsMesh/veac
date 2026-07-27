use std::collections::BTreeMap;

use crate::authoring::{ItemDecl, LayerDecl, LayerKind, MappingDecl, SequenceDecl, SourceDecl};
use veac_ir::{
    Clip, ClipSource, PlacementMode, Sequence, SequenceSettings, Track, TrackKind, TrackRouting,
    TrackState,
};

use super::context::Context;
use super::{ids, mapping, modifier, source, timeline_audio, value};

pub(super) fn sequence(
    ctx: &mut Context,
    declaration: &SequenceDecl,
    settings: &SequenceSettings,
) -> Option<Sequence> {
    let tracks = declaration
        .layers
        .iter()
        .enumerate()
        .map(|(index, layer)| track(ctx, layer, index))
        .collect::<Option<Vec<_>>>()?;
    Some(Sequence {
        id: ids::sequence(ctx, &declaration.id)?,
        name: declaration.id.value.clone(),
        settings: settings.clone(),
        tracks,
        applies: Vec::new(),
        metadata: BTreeMap::new(),
    })
}

fn track(ctx: &mut Context, declaration: &LayerDecl, index: usize) -> Option<Track> {
    let order = match &declaration.order {
        Some(value) => value::integer_i32(ctx, value, "layer order")?,
        None => i32::try_from(index).ok()?,
    };
    let kind = track_kind(declaration.kind);
    let clips = declaration
        .items
        .iter()
        .map(|item| clip(ctx, item, kind, order))
        .collect::<Option<Vec<_>>>()?;
    Some(Track {
        id: ids::track(ctx, &declaration.id)?,
        kind,
        order,
        placement_mode: PlacementMode::Free,
        state: TrackState {
            enabled: true,
            muted: false,
            solo: false,
            locked: false,
        },
        routing: match &declaration.route_bus {
            Some(bus) => TrackRouting::AudioBus {
                bus_id: ids::bus_id(ctx, bus)?,
            },
            None => TrackRouting::Default,
        },
        clips,
    })
}

fn clip(ctx: &mut Context, value: &ItemDecl, kind: TrackKind, z: i32) -> Option<Clip> {
    let (replaceable, template_editable_text) =
        super::template_slot::lower(ctx, value.template_slot.as_ref())?;
    let source = clip_source(ctx, value)?;
    let source_mapping = source_mapping(ctx, value, &source)?;
    let modifiers = modifier::lower(ctx, &value.modifiers, z)?;
    let audio = timeline_audio::resolve(ctx, value, kind, modifiers.audio)?;
    if kind == TrackKind::Audio && has_visual_modifier(value) {
        ctx.error(
            "AUTHORING_LOWER_TRACK_COMPONENT",
            "audio-layer items cannot contain visual modifiers",
            value.span,
        );
    }
    Some(Clip {
        id: ids::item(ctx, &value.id)?,
        enabled: true,
        record_range: value::range(ctx, &value.record.at, &value.record.duration)?,
        source,
        source_mapping,
        visual: (kind != TrackKind::Audio).then_some(modifiers.visual),
        audio,
        effects: modifiers.effects,
        replaceable,
        template_editable_text,
        metadata: BTreeMap::new(),
    })
}

fn clip_source(ctx: &mut Context, value: &ItemDecl) -> Option<ClipSource> {
    match (&value.source, &value.mapping) {
        (SourceDecl::Media { resource, .. }, Some(MappingDecl::Freeze { source, .. })) => {
            Some(ClipSource::FreezeFrame {
                material_id: ids::material(ctx, &resource.id)?,
                source_time: value::time(ctx, source)?,
            })
        }
        _ => source::lower(ctx, &value.source),
    }
}

fn source_mapping(
    ctx: &mut Context,
    value: &ItemDecl,
    source: &ClipSource,
) -> Option<Option<veac_ir::SourceMapping>> {
    if matches!(
        source,
        ClipSource::Media { .. } | ClipSource::Sequence { .. }
    ) {
        return mapping::lower(ctx, value.mapping.as_ref(), &value.record.duration).map(Some);
    }
    if value.mapping.is_some() && !matches!(source, ClipSource::FreezeFrame { .. }) {
        ctx.error(
            "AUTHORING_LOWER_MAPPING_SOURCE",
            "only media and sequence sources accept source-time mapping",
            value.mapping.as_ref().map_or(value.span, mapping_span),
        );
        return None;
    }
    Some(None)
}

fn mapping_span(value: &MappingDecl) -> crate::authoring::Span {
    match value {
        MappingDecl::Linear { span, .. }
        | MappingDecl::Curve { span, .. }
        | MappingDecl::Freeze { span, .. } => *span,
    }
}

fn has_visual_modifier(value: &ItemDecl) -> bool {
    value.modifiers.iter().any(|modifier| {
        matches!(
            modifier,
            crate::authoring::ModifierDecl::Layout(_)
                | crate::authoring::ModifierDecl::Transform(_)
                | crate::authoring::ModifierDecl::Composite(_)
        )
    })
}

fn track_kind(value: LayerKind) -> TrackKind {
    match value {
        LayerKind::Video => TrackKind::Video,
        LayerKind::Audio => TrackKind::Audio,
        LayerKind::Visual => TrackKind::Visual,
        LayerKind::Caption => TrackKind::Caption,
    }
}
