use veac_artifact::{artifact_key, ArtifactKind, ContentDigest};
use veac_ir::{
    ClipSource, EditOperation, EditOutcome, ItemId, RelationKind, StructureEdit, TrackMatteMode,
};

use crate::*;

use super::support::*;

#[test]
fn segmentation_materializes_and_applies_an_exact_temporal_matte() {
    let project = project();
    let (request, response) = exchange(Capability::Segmentation);
    let ProviderOutput::Segmentation(result) = &response.output else {
        unreachable!()
    };
    let proposal = propose_edit(
        &project,
        &request,
        &response,
        &matte_context(Capability::Segmentation, &result.matte),
    )
    .unwrap();
    assert_eq!(proposal.batch.operations.len(), 3);
    assert!(matches!(
        &proposal.batch.operations[0],
        EditOperation::EditStructure {
            edit: StructureEdit::InsertMaterial { material, .. }
        } if material.id.as_str() == "med_provider_matte"
    ));
    assert!(matches!(
        &proposal.batch.operations[1],
        EditOperation::InsertClip { clip, .. }
            if matches!(&clip.source, ClipSource::Media { material_id }
                if material_id.as_str() == "med_provider_matte")
    ));
    let EditOutcome::Applied {
        project: applied, ..
    } = veac_ir::apply_edit_batch(&project, &proposal.batch)
    else {
        unreachable!()
    };
    assert!(matches!(
        &applied.project.relations[0].kind,
        RelationKind::Matte { producer, consumer, parameters }
            if producer.item_id().unwrap().as_str() == "itm_provider_matte"
                && consumer.item_id().unwrap().as_str() == "itm_video"
                && parameters.mode == TrackMatteMode::Alpha
    ));
}

#[test]
fn matte_output_uses_the_same_executable_application_path() {
    let project = project();
    let (request, response) = exchange(Capability::Matte);
    let ProviderOutput::Matte(result) = &response.output else {
        unreachable!()
    };
    let context = matte_context(Capability::Matte, &result.matte);
    let proposal = propose_edit(&project, &request, &response, &context).unwrap();
    assert!(matches!(
        &proposal.batch.operations[2],
        EditOperation::EditStructure {
            edit: StructureEdit::InsertRelation { relation }
        } if matches!(&relation.kind, RelationKind::Matte { producer, consumer, .. }
            if producer.item_id().unwrap().as_str() == "itm_provider_matte"
                && consumer.item_id().unwrap().as_str() == "itm_video")
    ));
    assert!(matches!(
        veac_ir::apply_edit_batch(&project, &proposal.batch),
        EditOutcome::Applied { .. }
    ));
}

#[test]
fn matte_application_rejects_non_exact_or_locked_bindings() {
    let (request, response) = exchange(Capability::Segmentation);
    let ProviderOutput::Segmentation(result) = &response.output else {
        unreachable!()
    };
    let mut range = matte_context(Capability::Segmentation, &result.matte);
    matte_mut(&mut range).matte.record_range.start = time(1);
    assert_invalid(propose_edit(&project(), &request, &response, &range));

    let mut duplicate = matte_context(Capability::Segmentation, &result.matte);
    matte_mut(&mut duplicate).matte.clip_id = ItemId::new("itm_video").unwrap();
    assert_invalid(propose_edit(&project(), &request, &response, &duplicate));

    let mut wrong_source = matte_context(Capability::Segmentation, &result.matte);
    matte_mut(&mut wrong_source).target_clip_id = ItemId::new("itm_visual").unwrap();
    assert_invalid(propose_edit(&project(), &request, &response, &wrong_source));

    let mut short = matte_context(Capability::Segmentation, &result.matte);
    let probe = matte_mut(&mut short)
        .matte
        .material
        .material
        .probe
        .as_mut()
        .unwrap();
    probe.container_duration = Some(time(10));
    probe.streams[0].duration = Some(time(10));
    assert_invalid(propose_edit(&project(), &request, &response, &short));

    let mut locked = project();
    locked.project.sequences[0].tracks[0].state.locked = true;
    assert_invalid(propose_edit(
        &locked,
        &request,
        &response,
        &matte_context(Capability::Segmentation, &result.matte),
    ));
}

#[test]
fn matte_application_rejects_wrong_artifacts_and_tampering_atomically() {
    let (request, mut response) = exchange(Capability::Segmentation);
    let artifact = match &mut response.output {
        ProviderOutput::Segmentation(result) => {
            result.matte.descriptor.kind = ArtifactKind::RenderSegment;
            result.matte.record.key = artifact_key(&result.matte.descriptor).unwrap();
            result.matte.clone()
        }
        _ => unreachable!(),
    };
    assert_invalid(propose_edit(
        &project(),
        &request,
        &response,
        &matte_context(Capability::Segmentation, &artifact),
    ));

    let (request, response) = exchange(Capability::Matte);
    let ProviderOutput::Matte(result) = &response.output else {
        unreachable!()
    };
    let proposal = propose_edit(
        &project(),
        &request,
        &response,
        &matte_context(Capability::Matte, &result.matte),
    )
    .unwrap();
    let mut evidence = proposal.clone();
    let ProposalEvidence::AppliedTrackMatte { artifact_key, .. } = &mut evidence.evidence[2] else {
        unreachable!()
    };
    *artifact_key = ContentDigest::sha256(b"tampered");
    assert!(canonical_edit_proposal_bytes(&evidence).is_err());

    let project = project();
    let mut batch = proposal.batch;
    let EditOperation::EditStructure {
        edit: StructureEdit::InsertRelation { relation },
    } = &mut batch.operations[2]
    else {
        unreachable!()
    };
    let RelationKind::Matte { consumer, .. } = &mut relation.kind else {
        unreachable!()
    };
    *consumer = veac_ir::RelationEndpoint::item(ItemId::new("itm_missing").unwrap());
    assert!(matches!(
        veac_ir::apply_edit_batch(&project, &batch),
        EditOutcome::Rejected { .. }
    ));
    assert_eq!(project.project.materials.len(), 2);
}

fn matte_mut(value: &mut ApplicationContext) -> &mut MatteApplication {
    match value {
        ApplicationContext::Segmentation(value) | ApplicationContext::Matte(value) => value,
        _ => unreachable!(),
    }
}

fn assert_invalid(value: ProviderResult<ProviderEditProposal>) {
    assert_eq!(value.unwrap_err().kind, ProviderErrorKind::InvalidContract);
}
