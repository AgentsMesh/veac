use veac_artifact::ArtifactKind;
use veac_ir::{
    Clip, ClipSource, EditOperation, ItemId, MaterialKind, MatteRelationParameters,
    ProjectEnvelope, Rational, RationalTime, Relation, RelationEndpoint, RelationId, RelationKind,
    Sequence, SequenceId, SourceMapping, StructureEdit, TimeRange, Track, TrackKind,
    TrackMatteMode,
};

use super::media::{self, RequiredStream};
use super::support::{self, BuiltApplication};
use crate::{
    MatteClipInsertion, OperationBinding, ProposalEvidence, ProviderArtifact, ProviderError,
    ProviderErrorKind, ProviderResult,
};

pub(super) fn target<'a>(
    project: &'a ProjectEnvelope,
    id: &ItemId,
) -> ProviderResult<(&'a Sequence, &'a Track, &'a Clip)> {
    let found = project.project.sequences.iter().find_map(|sequence| {
        sequence.tracks.iter().find_map(|track| {
            track
                .clips
                .iter()
                .find(|clip| clip.id == *id)
                .map(|clip| (sequence, track, clip))
        })
    });
    let (sequence, track, clip) = found.ok_or_else(|| invalid("visual target does not exist"))?;
    if track.state.locked
        || !track.state.enabled
        || !clip.enabled
        || clip.visual.is_none()
        || !matches!(track.kind, TrackKind::Video | TrackKind::Visual)
    {
        return support::invalid("visual target must be enabled and unlocked");
    }
    Ok((sequence, track, clip))
}

pub(super) fn validate_matte<'a>(
    project: &'a ProjectEnvelope,
    artifact: &ProviderArtifact,
    insertion: &MatteClipInsertion,
    sequence: &'a Sequence,
    expected_range: TimeRange,
) -> ProviderResult<(&'a Track, TimeRange, RationalTime)> {
    if artifact.kind() != ArtifactKind::Matte
        || insertion.sequence_id != sequence.id
        || insertion.before_id.is_some() && insertion.after_id.is_some()
        || item_exists(project, &insertion.clip_id)
    {
        return support::invalid("matte clip binding is not unique or canonical");
    }
    let track = sequence
        .tracks
        .iter()
        .find(|track| track.id == insertion.track_id)
        .ok_or_else(|| invalid("matte target track does not exist"))?;
    if track.state.locked
        || !track.state.enabled
        || !matches!(track.kind, TrackKind::Video | TrackKind::Visual)
    {
        return support::invalid("matte target must be an enabled unlocked visual track");
    }
    let record_range = media::normalized_range(project, insertion.record_range)?;
    let source_start = media::normalized_time(project, insertion.source_start)?;
    if record_range != expected_range {
        return support::invalid("matte clip must exactly cover its visual target");
    }
    media::output_material(
        project,
        artifact,
        &insertion.material,
        MaterialKind::Video,
        RequiredStream::Video,
        source_start,
        record_range.duration,
    )?;
    Ok((track, record_range, source_start))
}

pub(super) fn append_matte(
    built: &mut BuiltApplication,
    insertion: &MatteClipInsertion,
    artifact: &ProviderArtifact,
    record_range: TimeRange,
    source_start: RationalTime,
) -> ProviderResult<()> {
    media::append_material(built, &insertion.material, artifact)?;
    let rate = Rational::new(1, 1).map_err(|error| {
        ProviderError::with_source(
            ProviderErrorKind::InvalidContract,
            "provider matte source rate is invalid",
            error,
        )
    })?;
    let operation = EditOperation::InsertClip {
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
            visual: Some(insertion.visual.clone()),
            audio: None,
            effects: Vec::new(),
            replaceable: None,
            template_editable_text: false,
            authorship: None,
        }),
        before_id: insertion.before_id.clone(),
        after_id: insertion.after_id.clone(),
    };
    let index = support::index(built.operations.len())?;
    built.evidence.push(ProposalEvidence::ArtifactVisualClip {
        operation: OperationBinding::new(index, &operation)?,
        artifact_key: artifact.record.key.clone(),
        artifact_role: artifact.role.as_str().to_owned(),
        material_id: insertion.material.material.id.clone(),
    });
    built.operations.push(operation);
    Ok(())
}

pub(super) fn append_track_matte(
    built: &mut BuiltApplication,
    sequence_id: &SequenceId,
    target_id: &ItemId,
    matte_clip_id: &ItemId,
    mode: TrackMatteMode,
    invert: bool,
    artifact: &ProviderArtifact,
) -> ProviderResult<()> {
    let relation_id = RelationId::new(format!("rel_matte_{}", target_id.as_str()))
        .map_err(|_| invalid("matte relation ID is invalid"))?;
    let operation = EditOperation::EditStructure {
        edit: StructureEdit::InsertRelation {
            relation: Relation {
                id: relation_id,
                sequence_id: sequence_id.clone(),
                kind: RelationKind::Matte {
                    producer: RelationEndpoint::item(matte_clip_id.clone()),
                    consumer: RelationEndpoint::item(target_id.clone()),
                    parameters: MatteRelationParameters { mode, invert },
                },
            },
        },
    };
    let index = support::index(built.operations.len())?;
    built.evidence.push(ProposalEvidence::AppliedTrackMatte {
        operation: OperationBinding::new(index, &operation)?,
        artifact_key: artifact.record.key.clone(),
        artifact_role: artifact.role.as_str().to_owned(),
        matte_clip_id: matte_clip_id.clone(),
    });
    built.operations.push(operation);
    Ok(())
}

fn item_exists(project: &ProjectEnvelope, id: &ItemId) -> bool {
    project
        .project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
        .any(|clip| clip.id == *id)
}

fn invalid(message: &str) -> ProviderError {
    ProviderError::new(ProviderErrorKind::InvalidContract, message)
}
