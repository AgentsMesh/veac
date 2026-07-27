use veac_ir::{EditOperation, HashAlgorithm, MulticamAngle, MulticamSync, ProjectEnvelope};

use super::support::{self, BuiltApplication};
use crate::{
    MulticamSyncApplication, MulticamSyncRequest, MulticamSyncResult, OperationBinding,
    ProposalEvidence, ProviderResult,
};

pub(super) fn build(
    project: &ProjectEnvelope,
    request: &MulticamSyncRequest,
    result: &MulticamSyncResult,
    context: &MulticamSyncApplication,
) -> ProviderResult<BuiltApplication> {
    if request.basis != result.basis
        || request.reference_angle_id != result.reference_angle_id
        || request.angles.len() != result.offsets.len()
    {
        return support::invalid("multicam sync result does not match its request");
    }
    let group = project
        .project
        .multicam_groups
        .iter()
        .find(|value| value.id == context.group_id)
        .ok_or_else(|| invalid_error("multicam sync target group does not exist"))?;
    if group.angles.len() != result.offsets.len() {
        return support::invalid("multicam sync result does not cover the target group");
    }
    let mut angles = Vec::with_capacity(group.angles.len());
    for ((current, input), offset) in group
        .angles
        .iter()
        .zip(&request.angles)
        .zip(&result.offsets)
    {
        validate_angle(project, current, input, offset, request)?;
        angles.push(MulticamAngle {
            id: current.id.clone(),
            material_id: current.material_id.clone(),
            source_offset: super::super::time::convert(
                offset.source_offset,
                project.project.timebase,
            )?,
        });
    }
    let operation = EditOperation::SetMulticamGroup {
        group_id: group.id.clone(),
        sync: MulticamSync {
            basis: result.basis,
            reference_angle_id: result.reference_angle_id.clone(),
        },
        angles,
    };
    let evidence = ProposalEvidence::MulticamSync {
        operation: OperationBinding::new(0, &operation)?,
        group_id: group.id.clone(),
        angle_indices: (0..result.offsets.len())
            .map(support::index)
            .collect::<ProviderResult<Vec<_>>>()?,
    };
    Ok(BuiltApplication::new(vec![operation], vec![evidence]))
}

fn validate_angle(
    project: &ProjectEnvelope,
    current: &MulticamAngle,
    input: &crate::MulticamSyncInput,
    offset: &crate::MulticamSyncOffset,
    request: &MulticamSyncRequest,
) -> ProviderResult<()> {
    if current.id != input.angle_id
        || current.id != offset.angle_id
        || current.material_id != input.material_id
        || current.material_id != offset.material_id
    {
        return support::invalid("multicam sync angle identity does not match the target group");
    }
    if offset.source_offset > request.maximum_offset {
        return support::invalid("multicam sync offset exceeds the requested limit");
    }
    let material = project
        .project
        .materials
        .iter()
        .find(|value| value.id == current.material_id)
        .ok_or_else(|| invalid_error("multicam sync angle material does not exist"))?;
    let identity_matches = material.identity.as_ref().is_some_and(|identity| {
        identity.algorithm == HashAlgorithm::Sha256 && identity.digest == input.media.content.value
    });
    let stream_matches = material.probe.as_ref().is_some_and(|probe| {
        let selected = match request.basis {
            veac_ir::MulticamSyncBasis::Audio => probe.selected_audio_stream,
            veac_ir::MulticamSyncBasis::Timecode => probe.selected_video_stream,
            veac_ir::MulticamSyncBasis::Manual => None,
        };
        selected.map(|value| value.global_index) == input.media.stream_index
    });
    if !identity_matches || !stream_matches {
        return support::invalid("multicam sync input is not bound to the angle material");
    }
    Ok(())
}

fn invalid_error(message: &str) -> crate::ProviderError {
    crate::ProviderError::new(crate::ProviderErrorKind::InvalidContract, message)
}
