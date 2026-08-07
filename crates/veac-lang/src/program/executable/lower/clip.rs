use crate::program::expression::runtime::domain_graph::{FrozenDomainGraph, FrozenEntity};
use crate::program::{DomainOperationId as Op, DomainType};
use veac_ir::{Clip, ClipSource, TrackKind};

use super::error::ExecutableLowerError;
use super::{audio, defaults, effect, id, source_timing, template, time, value, visual};

mod source;
mod text;

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    entity: FrozenEntity<'_>,
    timebase: u32,
    z_index: i32,
    track_kind: TrackKind,
) -> Result<Clip, ExecutableLowerError> {
    let operands = value::entity_operands(entity, DomainType::Item, Op::Item)?;
    if operands.len() != 5 {
        return Err(malformed());
    }
    reject_pending_attachments(entity)?;
    let path = entity.logical_path().ok_or_else(malformed)?;
    let record_range = time::range(graph, &operands[2], timebase)?;
    let source = source::lower(graph, &operands[3], timebase, record_range.duration, &path)?;
    reject_text_track(&source.value, track_kind)?;
    let source_mapping = source_timing::lower(graph, &operands[4], timebase, source.timed)?;
    let template = template_contract(graph, entity, timebase)?;
    Ok(Clip {
        id: id::item(&path),
        enabled: item_enabled(graph, &operands[1])?,
        record_range,
        source: source.value,
        source_mapping,
        visual: visual_style(graph, entity, timebase, &path, z_index, track_kind)?,
        audio: audio_style(graph, entity, timebase, &path)?,
        effects: effect::lower(graph, entity, timebase, &path)?,
        replaceable: template.as_ref().map(|value| value.constraint.clone()),
        template_editable_text: template.is_some_and(|value| value.editable_text),
        authorship: Some(super::metadata::entity(entity)?),
    })
}

fn reject_text_track(
    source: &ClipSource,
    track_kind: TrackKind,
) -> Result<(), ExecutableLowerError> {
    let compatible = match source {
        ClipSource::Text { .. } => track_kind == TrackKind::Visual,
        ClipSource::Caption { .. } => track_kind == TrackKind::Caption,
        _ => true,
    };
    compatible.then_some(()).ok_or_else(super::text::invalid)
}

fn item_enabled(
    graph: &FrozenDomainGraph,
    source: &crate::program::expression::Value,
) -> Result<bool, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::ItemEnabled, []) => Ok(true),
        (Op::ItemDisabled, []) => Ok(false),
        _ => Err(malformed()),
    }
}

fn reject_pending_attachments(entity: FrozenEntity<'_>) -> Result<(), ExecutableLowerError> {
    let _ = entity;
    Ok(())
}

fn template_contract(
    graph: &FrozenDomainGraph,
    entity: FrozenEntity<'_>,
    timebase: u32,
) -> Result<Option<template::LoweredTemplate>, ExecutableLowerError> {
    let Some(values) = entity.latest_update(Op::ItemWithTemplate) else {
        return Ok(None);
    };
    let [_, contract] = values else {
        return Err(malformed());
    };
    Ok(Some(template::lower(graph, contract, timebase)?))
}

fn audio_style(
    graph: &FrozenDomainGraph,
    entity: FrozenEntity<'_>,
    timebase: u32,
    scope: &[&str],
) -> Result<Option<veac_ir::AudioProperties>, ExecutableLowerError> {
    let Some(values) = entity.latest_update(Op::ItemWithAudio) else {
        return Ok(None);
    };
    let [_, style] = values else {
        return Err(malformed());
    };
    Ok(Some(audio::lower(graph, style, timebase, scope)?))
}

fn visual_style(
    graph: &FrozenDomainGraph,
    entity: FrozenEntity<'_>,
    timebase: u32,
    scope: &[&str],
    z_index: i32,
    track_kind: TrackKind,
) -> Result<Option<veac_ir::VisualProperties>, ExecutableLowerError> {
    let Some(values) = entity.latest_update(Op::ItemWithVisual) else {
        return Ok((track_kind != TrackKind::Audio).then(|| defaults::visual(z_index)));
    };
    let [_, style] = values else {
        return Err(malformed());
    };
    Ok(Some(visual::lower(graph, style, timebase, scope)?))
}

fn malformed() -> ExecutableLowerError {
    value::graph("an executable item has invalid typed source or timing topology")
}
