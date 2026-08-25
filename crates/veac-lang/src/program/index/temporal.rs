use crate::source_edit::{BodySite, SourceNodeRef, SourceTemporalProperty};

use super::super::diagnostic::Diagnostic;
use super::super::model::{SurfaceFile, TemporalDecl, TemporalProperty, TemporalTarget};
use super::SourceIndex;

pub(super) fn index(index: &mut SourceIndex, file: &SurfaceFile) -> Result<(), Diagnostic> {
    for declaration in &file.temporal {
        let target = target(&file.path, declaration);
        index.register(&file.path, target.clone(), declaration.syntax.span)?;
        index.insert_declaration(
            &file.path,
            target.clone(),
            crate::source_edit::DeclarationSite::TemporalDeclaration,
            file.source(),
            declaration.syntax.span,
        )?;
        index.insert_body(
            &file.path,
            target,
            BodySite::TemporalAnimation {
                property: property(declaration.property),
            },
            file.syntax.slice_text(&declaration.body.syntax),
            declaration.body.span,
        )?;
    }
    Ok(())
}

pub(super) fn target(module: &str, declaration: &TemporalDecl) -> SourceNodeRef {
    let property = property(declaration.property);
    match &declaration.target {
        TemporalTarget::Clip(path) | TemporalTarget::Text(path) => SourceNodeRef::temporal(
            module,
            &path.project,
            &path.sequence,
            &path.layer,
            &path.item,
            property,
        ),
        TemporalTarget::ClipMask { clip, mask_index } => {
            SourceNodeRef::temporal_clip_mask(module, clip.segments(), *mask_index, property)
        }
        TemporalTarget::ClipEffect {
            clip,
            effect,
            parameter,
        } => SourceNodeRef::temporal_clip_effect(
            module,
            clip.segments(),
            effect,
            parameter,
            property,
        ),
        TemporalTarget::Apply(path) => {
            SourceNodeRef::temporal_apply(module, path.segments(), property)
        }
        TemporalTarget::ApplyMask { apply, mask_index } => {
            SourceNodeRef::temporal_apply_mask(module, apply.segments(), *mask_index, property)
        }
        TemporalTarget::ApplyEffect {
            apply,
            stage,
            effect,
            parameter,
        } => SourceNodeRef::temporal_apply_effect(
            module,
            apply.segments(),
            [stage, effect, parameter],
            property,
        ),
    }
}

pub(super) fn property(value: TemporalProperty) -> SourceTemporalProperty {
    match value {
        TemporalProperty::VisualPosition => SourceTemporalProperty::VisualPosition,
        TemporalProperty::VisualScale => SourceTemporalProperty::VisualScale,
        TemporalProperty::VisualRotation => SourceTemporalProperty::VisualRotation,
        TemporalProperty::VisualCrop => SourceTemporalProperty::VisualCrop,
        TemporalProperty::VisualOpacity => SourceTemporalProperty::VisualOpacity,
        TemporalProperty::AudioGain => SourceTemporalProperty::AudioGain,
        TemporalProperty::AudioPan => SourceTemporalProperty::AudioPan,
        TemporalProperty::MaskPosition => SourceTemporalProperty::MaskPosition,
        TemporalProperty::MaskScale => SourceTemporalProperty::MaskScale,
        TemporalProperty::MaskRotation => SourceTemporalProperty::MaskRotation,
        TemporalProperty::MaskFeather => SourceTemporalProperty::MaskFeather,
        TemporalProperty::MaskExpansion => SourceTemporalProperty::MaskExpansion,
        TemporalProperty::TextPosition => SourceTemporalProperty::TextPosition,
        TemporalProperty::TextScale => SourceTemporalProperty::TextScale,
        TemporalProperty::TextRotation => SourceTemporalProperty::TextRotation,
        TemporalProperty::TextReveal => SourceTemporalProperty::TextReveal,
        TemporalProperty::TextHighlightProgress => SourceTemporalProperty::TextHighlightProgress,
        TemporalProperty::TextOpacity => SourceTemporalProperty::TextOpacity,
        TemporalProperty::EffectParameter => SourceTemporalProperty::EffectParameter,
        TemporalProperty::ApplyOpacity => SourceTemporalProperty::ApplyOpacity,
    }
}
