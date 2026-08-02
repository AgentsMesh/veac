mod audio;
mod color;
mod delivery;
mod text;

use crate::source_edit::{ExpressionSite, SourceNodeRef, SourcePresetKind};

use super::super::diagnostic::Diagnostic;
use super::super::model::{PresetDecl, PresetKind, SurfaceFile};
use super::item;
use super::syntax::{self, Block, Entry};
use super::SourceIndex;

pub(super) fn index(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    preset: &PresetDecl,
) -> Result<(), Diagnostic> {
    let preset_kind = public_kind(preset.kind);
    let target = SourceNodeRef::preset(&file.path, preset_kind, &preset.name);
    let declaration = syntax::parse(&file.path, &file.source, preset.span)?;
    if let Some(span) = declaration
        .entries
        .first()
        .and_then(|value| value.word_span(2))
    {
        index.register(&file.path, target.clone(), span)?;
    }
    let body = syntax::parse(&file.path, &file.source, preset.body.content_span)?;
    match preset.kind {
        PresetKind::TextStyle => text::style(index, file, target, &body),
        PresetKind::TextLayout => text::layout(index, file, target, &body),
        PresetKind::ModifierStack => modifier_stack(index, file, preset, &body),
        PresetKind::EffectPipeline => effect_pipeline(index, file, preset, &body),
        PresetKind::ColorPipeline => color::index(index, file, target, &body),
        PresetKind::AudioProcessors => audio::index(index, file, preset, &body),
        PresetKind::DeliveryProfile => delivery::index(index, file, preset, &body),
    }
}

fn modifier_stack(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    preset: &PresetDecl,
    body: &Block,
) -> Result<(), Diagnostic> {
    for entry in &body.entries {
        if !matches!(
            entry.word(0),
            Some(
                "layout"
                    | "transform"
                    | "composite"
                    | "surface"
                    | "mask"
                    | "audio"
                    | "color"
                    | "effect"
            )
        ) {
            continue;
        }
        let Some(id) = entry.id(1) else { continue };
        let target = SourceNodeRef::preset_modifier(&file.path, &preset.name, id);
        item::parameters(index, file, target, entry, 1)?;
    }
    Ok(())
}

fn effect_pipeline(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    preset: &PresetDecl,
    body: &Block,
) -> Result<(), Diagnostic> {
    for entry in body
        .entries
        .iter()
        .filter(|value| value.word(0) == Some("stage"))
    {
        let Some(id) = entry.id(2) else { continue };
        let target = SourceNodeRef::preset_stage(&file.path, &preset.name, id);
        item::parameters(index, file, target, entry, 2)?;
    }
    Ok(())
}

pub(super) fn insert(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: SourceNodeRef,
    site: ExpressionSite,
    entry: &Entry,
    skip: usize,
) -> Result<(), Diagnostic> {
    let Some((source, span)) = entry.expression(&file.source, skip) else {
        return Ok(());
    };
    index.insert(&file.path, target, site, source, span)
}

fn public_kind(value: PresetKind) -> SourcePresetKind {
    match value {
        PresetKind::TextStyle => SourcePresetKind::TextStyle,
        PresetKind::TextLayout => SourcePresetKind::TextLayout,
        PresetKind::ModifierStack => SourcePresetKind::ModifierStack,
        PresetKind::EffectPipeline => SourcePresetKind::EffectPipeline,
        PresetKind::ColorPipeline => SourcePresetKind::ColorPipeline,
        PresetKind::AudioProcessors => SourcePresetKind::AudioProcessors,
        PresetKind::DeliveryProfile => SourcePresetKind::DeliveryProfile,
    }
}
