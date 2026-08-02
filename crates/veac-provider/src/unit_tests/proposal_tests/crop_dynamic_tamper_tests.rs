use veac_ir::{Animatable, EditOperation, VisualProperty};

use crate::*;

use super::crop_dynamic_tests::dynamic_fixture;

#[test]
fn crop_curve_evidence_rejects_source_time_prefix_target_and_value_tampering() {
    let (project, request, response, context) = dynamic_fixture();
    let proposal = propose_edit(&project, &request, &response, &context).unwrap();

    let mut source = proposal.clone();
    let ProviderOutput::AutoReframe(output) = &mut source.source_output else {
        unreachable!()
    };
    output.crops[1].rect.x = 0.45;
    assert!(canonical_edit_proposal_bytes(&source).is_err());

    for mutate in [tamper_time as fn(&mut ProposalEvidence), tamper_prefix] {
        let mut value = proposal.clone();
        mutate(&mut value.evidence[0]);
        assert!(canonical_edit_proposal_bytes(&value).is_err());
    }

    let mut target = proposal.clone();
    let EditOperation::SetVisualProperty { clip_id, .. } = &mut target.batch.operations[0] else {
        unreachable!()
    };
    *clip_id = veac_ir::ItemId::new("itm_other").unwrap();
    rebind(&mut target, 0);
    assert!(canonical_edit_proposal_bytes(&target).is_err());

    let mut operation = proposal;
    let EditOperation::SetVisualProperty {
        property: VisualProperty::Crop(Some(Animatable::Keyframes { keyframes })),
        ..
    } = &mut operation.batch.operations[0]
    else {
        unreachable!()
    };
    keyframes[0].value.x += 0.01;
    rebind(&mut operation, 0);
    assert!(canonical_edit_proposal_bytes(&operation).is_err());
}

fn tamper_time(value: &mut ProposalEvidence) {
    let ProposalEvidence::CropSamples { time, .. } = value else {
        unreachable!()
    };
    time.clip_local_origin = veac_ir::RationalTime::new(21, 100).unwrap();
}

fn tamper_prefix(value: &mut ProposalEvidence) {
    let ProposalEvidence::CropSamples {
        keyframe_id_prefix, ..
    } = value
    else {
        unreachable!()
    };
    keyframe_id_prefix.push_str("_tampered");
}

fn rebind(proposal: &mut ProviderEditProposal, index: usize) {
    let ProposalEvidence::CropSamples { operation, .. } = &mut proposal.evidence[index] else {
        unreachable!()
    };
    *operation = OperationBinding::new(index as u32, &proposal.batch.operations[index]).unwrap();
}
