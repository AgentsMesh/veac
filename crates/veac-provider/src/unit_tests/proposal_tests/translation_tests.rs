use veac_ir::{ClipSource, EditOperation, EditOutcome, ItemId};

use crate::*;

use super::support::*;

#[test]
fn translation_maps_exact_units_to_canonical_text_edits() {
    let project = project();
    let (request, response) = exchange(Capability::Translation);
    let proposal = propose_edit(&project, &request, &response, &translation_context()).unwrap();
    assert!(matches!(
        &proposal.batch.operations[0],
        EditOperation::SetText { clip_id, text }
            if clip_id.as_str() == "itm_text" && text == "ni hao"
    ));
    assert!(matches!(
        &proposal.evidence[0],
        ProposalEvidence::TranslationUnit { operation, unit_id }
            if operation.operation_index == 0 && unit_id == "unit-1"
    ));
    let EditOutcome::Applied {
        project: applied, ..
    } = veac_ir::apply_edit_batch(&project, &proposal.batch)
    else {
        unreachable!()
    };
    let ClipSource::Text { text, .. } = &applied.project.sequences[0].tracks[0].clips[0].source
    else {
        unreachable!()
    };
    assert_eq!(text, "ni hao");
}

#[test]
fn translation_rejects_binding_range_and_target_mismatches() {
    let (request, response) = exchange(Capability::Translation);
    let mut context = translation_context();
    if let ApplicationContext::Translation(value) = &mut context {
        value.bindings[0].unit_id = "wrong-unit".into();
    }
    assert_invalid(propose_edit(&project(), &request, &response, &context));
    if let ApplicationContext::Translation(value) = &mut context {
        value.bindings[0].unit_id = "unit-1".into();
        value.bindings[0].clip_id = ItemId::new("itm_visual").unwrap();
    }
    assert_invalid(propose_edit(&project(), &request, &response, &context));

    let (request, response) = exchange(Capability::Translation);
    let mut range_response = response;
    let ProviderOutput::Translation(result) = &mut range_response.output else {
        unreachable!()
    };
    result.units[0].range = Some(crate::test_support::range(1, 10));
    assert_invalid(propose_edit(
        &project(),
        &request,
        &range_response,
        &translation_context(),
    ));
}

#[test]
fn translation_rejects_locked_tracks_empty_results_and_duplicate_targets() {
    let (request, response) = exchange(Capability::Translation);
    let mut value = project();
    value.project.sequences[0].tracks[0].state.locked = true;
    assert_invalid(propose_edit(
        &value,
        &request,
        &response,
        &translation_context(),
    ));

    let (request, response) = exchange(Capability::Translation);
    let mut duplicate_response = response.clone();
    let ProviderOutput::Translation(result) = &mut duplicate_response.output else {
        unreachable!()
    };
    let mut second = result.units[0].clone();
    second.id = "unit-2".into();
    second.range = None;
    second.words.clear();
    result.units.push(second);
    let mut context = translation_context();
    let ApplicationContext::Translation(value) = &mut context else {
        unreachable!()
    };
    value.bindings.push(TranslationBinding {
        unit_id: "unit-2".into(),
        clip_id: ItemId::new("itm_text").unwrap(),
    });
    assert_invalid(propose_edit(
        &project(),
        &request,
        &duplicate_response,
        &context,
    ));

    let mut empty_response = response;
    let ProviderOutput::Translation(result) = &mut empty_response.output else {
        unreachable!()
    };
    result.units.clear();
    let context = translation_context();
    assert_invalid(propose_edit(
        &project(),
        &request,
        &empty_response,
        &context,
    ));
}

fn assert_invalid(value: ProviderResult<ProviderEditProposal>) {
    assert_eq!(value.unwrap_err().kind, ProviderErrorKind::InvalidContract);
}
