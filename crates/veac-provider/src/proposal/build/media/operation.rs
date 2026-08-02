use std::collections::BTreeMap;

use veac_ir::{
    Clip, ClipSource, EditOperation, Precondition, Rational, SourceMapping, StructureEdit,
};

use super::super::support::{self, BuiltApplication};
use crate::{
    AudioClipInsertion, MaterialInsertion, OperationBinding, ProposalEvidence, ProviderArtifact,
    ProviderResult, StemKind,
};

pub(crate) fn append_material(
    built: &mut BuiltApplication,
    insertion: &MaterialInsertion,
    artifact: &ProviderArtifact,
) -> ProviderResult<()> {
    let operation = EditOperation::EditStructure {
        edit: StructureEdit::InsertMaterial {
            material: Box::new(insertion.material.clone()),
            before_id: insertion.before_id.clone(),
            after_id: insertion.after_id.clone(),
        },
    };
    let index = support::index(built.operations.len())?;
    built.evidence.push(ProposalEvidence::ArtifactMaterial {
        operation: OperationBinding::new(index, &operation)?,
        artifact_key: artifact.record.key.clone(),
        artifact_role: artifact.role.clone(),
        material_id: insertion.material.id.clone(),
    });
    built.operations.push(operation);
    Ok(())
}

pub(crate) fn append_generated_clip(
    built: &mut BuiltApplication,
    insertion: &AudioClipInsertion,
    artifact: &ProviderArtifact,
    record_range: veac_ir::TimeRange,
    source_start: veac_ir::RationalTime,
) -> ProviderResult<()> {
    let operation = clip_operation(insertion, record_range, source_start)?;
    let index = support::index(built.operations.len())?;
    built.evidence.push(ProposalEvidence::GeneratedAudioClip {
        operation: OperationBinding::new(index, &operation)?,
        artifact_key: artifact.record.key.clone(),
        artifact_role: artifact.role.clone(),
        material_id: insertion.material.material.id.clone(),
    });
    built.operations.push(operation);
    Ok(())
}

pub(crate) fn append_stem_clip(
    built: &mut BuiltApplication,
    insertion: &AudioClipInsertion,
    artifact: &ProviderArtifact,
    kind: StemKind,
    label: &str,
    record_range: veac_ir::TimeRange,
    source_start: veac_ir::RationalTime,
) -> ProviderResult<()> {
    let operation = clip_operation(insertion, record_range, source_start)?;
    let index = support::index(built.operations.len())?;
    built.evidence.push(ProposalEvidence::SeparatedStemClip {
        operation: OperationBinding::new(index, &operation)?,
        artifact_key: artifact.record.key.clone(),
        artifact_role: artifact.role.clone(),
        material_id: insertion.material.material.id.clone(),
        kind,
        label: label.to_owned(),
    });
    built.operations.push(operation);
    Ok(())
}

pub(crate) fn append_replacement(
    built: &mut BuiltApplication,
    clip_id: &veac_ir::ItemId,
    insertion: &MaterialInsertion,
    artifact: &ProviderArtifact,
    source_start: veac_ir::RationalTime,
) -> ProviderResult<()> {
    let rate = Rational::new(1, 1).map_err(|error| {
        crate::ProviderError::with_source(
            crate::ProviderErrorKind::InvalidContract,
            "provider source rate is invalid",
            error,
        )
    })?;
    let operation = EditOperation::ReplaceSource {
        clip_id: clip_id.clone(),
        source: Box::new(ClipSource::Media {
            material_id: insertion.material.id.clone(),
        }),
        source_mapping: Some(SourceMapping::linear(source_start, rate)),
    };
    let index = support::index(built.operations.len())?;
    built.evidence.push(ProposalEvidence::ReplacedMedia {
        operation: OperationBinding::new(index, &operation)?,
        artifact_key: artifact.record.key.clone(),
        artifact_role: artifact.role.clone(),
        material_id: insertion.material.id.clone(),
    });
    built.operations.push(operation);
    Ok(())
}

fn clip_operation(
    insertion: &AudioClipInsertion,
    record_range: veac_ir::TimeRange,
    source_start: veac_ir::RationalTime,
) -> ProviderResult<EditOperation> {
    let rate = Rational::new(1, 1).map_err(|error| {
        crate::ProviderError::with_source(
            crate::ProviderErrorKind::InvalidContract,
            "provider source rate is invalid",
            error,
        )
    })?;
    Ok(EditOperation::InsertClip {
        sequence_id: insertion.sequence_id.clone(),
        track_id: insertion.track_id.clone(),
        clip: Box::new(Clip {
            id: insertion.clip_id.clone(),
            enabled: true,
            record_range,
            source: ClipSource::Media {
                material_id: insertion.material.material.id.clone(),
            },
            source_mapping: Some(SourceMapping::linear(source_start, rate)),
            visual: None,
            audio: Some(insertion.audio.clone()),
            effects: Vec::new(),
            replaceable: None,
            template_editable_text: false,
            metadata: BTreeMap::new(),
        }),
        before_id: insertion.before_id.clone(),
        after_id: insertion.after_id.clone(),
    })
}

pub(crate) fn add_track_precondition(built: &mut BuiltApplication, track: &veac_ir::Track) {
    if !built.preconditions.iter().any(
        |value| matches!(value, Precondition::TrackUnlocked { track_id } if *track_id == track.id),
    ) {
        built.preconditions.push(Precondition::TrackUnlocked {
            track_id: track.id.clone(),
        });
    }
}
