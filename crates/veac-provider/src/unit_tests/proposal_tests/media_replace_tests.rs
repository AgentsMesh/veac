use veac_ir::{ClipSource, EditOperation, EditOutcome, ItemId, StructureEdit};

use crate::*;

use super::support::*;

#[test]
fn denoise_inserts_material_and_replaces_audio_source_atomically() {
    let project = project();
    let (request, response) = exchange(Capability::Denoise);
    let ProviderOutput::Denoise(result) = &response.output else {
        unreachable!()
    };
    let proposal = propose_edit(
        &project,
        &request,
        &response,
        &denoise_context(&result.audio),
    )
    .unwrap();
    assert!(matches!(
        &proposal.batch.operations[0],
        EditOperation::EditStructure {
            edit: StructureEdit::InsertMaterial { material, .. }
        } if material.id.as_str() == "med_provider_denoise"
    ));
    assert!(matches!(
        &proposal.batch.operations[1],
        EditOperation::ReplaceSource {
            clip_id, source, source_mapping: Some(_),
        } if clip_id.as_str() == "itm_audio"
            && matches!(source.as_ref(), ClipSource::Media { material_id }
                if material_id.as_str() == "med_provider_denoise")
    ));
    let EditOutcome::Applied {
        project: applied, ..
    } = veac_ir::apply_edit_batch(&project, &proposal.batch)
    else {
        unreachable!()
    };
    assert!(applied.project.sequences[0].tracks[2].clips[0]
        .audio
        .is_some());
}

#[test]
fn removal_inserts_material_and_replaces_visual_source() {
    let project = project();
    let (request, response) = exchange(Capability::Removal);
    let ProviderOutput::Removal(result) = &response.output else {
        unreachable!()
    };
    let proposal = propose_edit(
        &project,
        &request,
        &response,
        &removal_context(&result.video),
    )
    .unwrap();
    let EditOutcome::Applied {
        project: applied, ..
    } = veac_ir::apply_edit_batch(&project, &proposal.batch)
    else {
        unreachable!()
    };
    assert!(matches!(
        &applied.project.sequences[0].tracks[3].clips[0].source,
        ClipSource::Media { material_id } if material_id.as_str() == "med_provider_removal"
    ));
    assert!(applied.project.sequences[0].tracks[3].clips[0]
        .visual
        .is_some());
}

#[test]
fn replacements_reject_locked_wrong_media_and_short_outputs() {
    let (request, response) = exchange(Capability::Denoise);
    let ProviderOutput::Denoise(result) = &response.output else {
        unreachable!()
    };
    let mut locked = project();
    locked.project.sequences[0].tracks[2].state.locked = true;
    assert_invalid(propose_edit(
        &locked,
        &request,
        &response,
        &denoise_context(&result.audio),
    ));

    let mut wrong = denoise_context(&result.audio);
    let ApplicationContext::Denoise(value) = &mut wrong else {
        unreachable!()
    };
    value.clip_id = ItemId::new("itm_video").unwrap();
    assert_invalid(propose_edit(&project(), &request, &response, &wrong));

    let mut short = denoise_context(&result.audio);
    let ApplicationContext::Denoise(value) = &mut short else {
        unreachable!()
    };
    let probe = value.material.material.probe.as_mut().unwrap();
    probe.container_duration = Some(time(10));
    probe.streams[0].duration = Some(time(10));
    assert_invalid(propose_edit(&project(), &request, &response, &short));
}

#[test]
fn edit_batch_rolls_back_inserted_material_when_replacement_fails() {
    let project = project();
    let (request, response) = exchange(Capability::Denoise);
    let ProviderOutput::Denoise(result) = &response.output else {
        unreachable!()
    };
    let proposal = propose_edit(
        &project,
        &request,
        &response,
        &denoise_context(&result.audio),
    )
    .unwrap();
    let mut batch = proposal.batch;
    let EditOperation::ReplaceSource { clip_id, .. } = &mut batch.operations[1] else {
        unreachable!()
    };
    *clip_id = ItemId::new("itm_missing").unwrap();
    assert!(matches!(
        veac_ir::apply_edit_batch(&project, &batch),
        EditOutcome::Rejected { .. }
    ));
    assert_eq!(project.project.materials.len(), 2);
}

fn assert_invalid(value: ProviderResult<ProviderEditProposal>) {
    assert_eq!(value.unwrap_err().kind, ProviderErrorKind::InvalidContract);
}
