use veac_artifact::ContentDigest;
use veac_ir::{ClipSource, EditOperation, ItemId, MaterialId, RelationKind, StructureEdit};

use crate::{ProviderArtifact, ProviderOutput, StemKind};

#[path = "artifact/retouch.rs"]
mod retouch_control;
pub(super) use retouch_control::matches as retouch;

pub(super) fn material(
    source: &ProviderOutput,
    operation: &EditOperation,
    key: &ContentDigest,
    role: &str,
    material_id: &MaterialId,
) -> bool {
    let Some(artifact) = find(source, key, role) else {
        return false;
    };
    matches!(
        operation,
        EditOperation::EditStructure {
            edit: StructureEdit::InsertMaterial { material, .. }
        } if material.id == *material_id
            && material.identity.as_ref().is_some_and(|identity| {
                identity.digest == artifact.record.content.value
                    && identity.algorithm == veac_ir::HashAlgorithm::Sha256
            })
    )
}

pub(super) fn generated_audio(
    source: &ProviderOutput,
    operation: &EditOperation,
    key: &ContentDigest,
    role: &str,
    material_id: &MaterialId,
) -> bool {
    if !matches!(
        source,
        ProviderOutput::TextToSpeech(_) | ProviderOutput::Dubbing(_)
    ) || find(source, key, role).is_none()
    {
        return false;
    }
    inserted_clip(operation, material_id)
}

pub(super) fn replacement(
    source: &ProviderOutput,
    operation: &EditOperation,
    key: &ContentDigest,
    role: &str,
    material_id: &MaterialId,
) -> bool {
    if !matches!(
        source,
        ProviderOutput::Denoise(_) | ProviderOutput::Removal(_)
    ) || find(source, key, role).is_none()
    {
        return false;
    }
    matches!(operation, EditOperation::ReplaceSource { source, .. }
        if matches!(source.as_ref(), ClipSource::Media { material_id: value }
            if value == material_id))
}

pub(super) fn stem(
    source: &ProviderOutput,
    operation: &EditOperation,
    key: &ContentDigest,
    role: &str,
    material_id: &MaterialId,
    kind: StemKind,
    label: &str,
) -> bool {
    let ProviderOutput::VocalSeparation(result) = source else {
        return false;
    };
    result.stems.iter().any(|stem| {
        stem.kind == kind
            && stem.label == label
            && stem.audio.record.key == *key
            && stem.audio.role == role
    }) && inserted_clip(operation, material_id)
}

pub(super) fn visual_clip(
    source: &ProviderOutput,
    operation: &EditOperation,
    key: &ContentDigest,
    role: &str,
    material_id: &MaterialId,
) -> bool {
    matches!(
        source,
        ProviderOutput::Segmentation(_) | ProviderOutput::Matte(_) | ProviderOutput::Retouch(_)
    ) && find(source, key, role).is_some()
        && inserted_clip(operation, material_id)
}

pub(super) fn track_matte(
    source: &ProviderOutput,
    operation: &EditOperation,
    key: &ContentDigest,
    role: &str,
    matte_clip_id: &ItemId,
) -> bool {
    matches!(
        source,
        ProviderOutput::Segmentation(_) | ProviderOutput::Matte(_) | ProviderOutput::Retouch(_)
    ) && find(source, key, role).is_some()
        && matches!(
            operation,
            EditOperation::EditStructure {
                edit: StructureEdit::InsertRelation { relation }
            } if matches!(&relation.kind, RelationKind::Matte { producer, .. }
                if producer.item_id() == Some(matte_clip_id))
        )
}

fn inserted_clip(operation: &EditOperation, material_id: &MaterialId) -> bool {
    matches!(
        operation,
        EditOperation::InsertClip { clip, .. }
            if matches!(&clip.source, ClipSource::Media { material_id: value } if value == material_id)
    )
}

fn find<'a>(
    source: &'a ProviderOutput,
    key: &ContentDigest,
    role: &str,
) -> Option<&'a ProviderArtifact> {
    source
        .artifacts()
        .into_iter()
        .find(|artifact| artifact.record.key == *key && artifact.role == role)
}
