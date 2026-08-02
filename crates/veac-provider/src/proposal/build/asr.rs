use std::collections::{BTreeMap, BTreeSet};

use veac_ir::{Clip, ClipSource, EditOperation, ItemId, Precondition, ProjectEnvelope, TrackKind};

use super::support::{self, BuiltApplication};
use crate::{
    AsrCaptionApplication, AsrResult, ProposalEvidence, ProviderError, ProviderErrorKind,
    ProviderResult,
};

pub(super) fn build(
    project: &ProjectEnvelope,
    result: &AsrResult,
    context: &AsrCaptionApplication,
) -> ProviderResult<BuiltApplication> {
    let target = target(project, context)?;
    if result.segments.is_empty() {
        return support::invalid("ASR output has no caption segments to propose");
    }
    let existing = item_ids(project);
    let mut operations = Vec::with_capacity(result.segments.len());
    let mut evidence = Vec::with_capacity(result.segments.len());
    for (index, segment) in result.segments.iter().enumerate() {
        let id = generated_id(&context.item_id_prefix, index)?;
        if existing.contains(&id) {
            return support::invalid("provider proposal item ID collides with the project");
        }
        let clip = caption(id, segment, context, project.project.timebase)?;
        let before_id = target
            .clips
            .iter()
            .find(|existing| existing.record_range.start > clip.record_range.start)
            .map(|existing| existing.id.clone());
        let operation_index = support::index(operations.len())?;
        let operation = EditOperation::InsertClip {
            sequence_id: context.sequence_id.clone(),
            track_id: context.track_id.clone(),
            clip: Box::new(clip),
            before_id,
            after_id: None,
        };
        evidence.push(ProposalEvidence::AsrSegment {
            operation: crate::OperationBinding::new(operation_index, &operation)?,
            segment_id: segment.id.clone(),
        });
        operations.push(operation);
    }
    let mut built = BuiltApplication::new(operations, evidence);
    built.preconditions.push(Precondition::TrackUnlocked {
        track_id: context.track_id.clone(),
    });
    Ok(built)
}

fn target<'a>(
    project: &'a ProjectEnvelope,
    context: &AsrCaptionApplication,
) -> ProviderResult<&'a veac_ir::Track> {
    let sequence = project
        .project
        .sequences
        .iter()
        .find(|value| value.id == context.sequence_id)
        .ok_or_else(|| invalid_error("provider proposal sequence does not exist"))?;
    let track = sequence
        .tracks
        .iter()
        .find(|value| value.id == context.track_id)
        .ok_or_else(|| invalid_error("provider proposal track does not exist"))?;
    if track.kind != TrackKind::Caption || track.state.locked {
        return support::invalid("provider proposal target must be an unlocked caption track");
    }
    Ok(track)
}

fn caption(
    id: ItemId,
    segment: &crate::TranscriptSegment,
    context: &AsrCaptionApplication,
    timebase: u32,
) -> ProviderResult<Clip> {
    Ok(Clip {
        id,
        enabled: true,
        record_range: super::super::time::range_to_timebase(segment.range, timebase)?,
        source: ClipSource::Caption {
            text: segment.text.clone(),
            speaker: None,
            style: context.style.clone(),
        },
        source_mapping: None,
        visual: Some(context.visual.clone()),
        audio: None,
        effects: Vec::new(),
        replaceable: None,
        template_editable_text: false,
        metadata: BTreeMap::new(),
    })
}

fn item_ids(project: &ProjectEnvelope) -> BTreeSet<ItemId> {
    project
        .project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
        .map(|clip| clip.id.clone())
        .collect()
}

fn generated_id(prefix: &str, index: usize) -> ProviderResult<ItemId> {
    ItemId::new(format!("{prefix}{:04}", index + 1)).map_err(|error| {
        ProviderError::with_source(
            ProviderErrorKind::InvalidContract,
            "provider proposal item ID prefix is invalid",
            error,
        )
    })
}

fn invalid_error(message: &str) -> ProviderError {
    ProviderError::new(ProviderErrorKind::InvalidContract, message)
}
