use std::collections::BTreeMap;

use veac_ir::*;

use crate::*;

use super::support::{exchange, project, retouch_context};

#[test]
fn retouch_builds_artifact_matte_and_item_set_apply() {
    let (project, request, response, context) = retouch_fixture();
    let proposal = proposal_result(&project, &request, &response, &context).unwrap();
    assert_eq!(proposal.batch.operations.len(), 4);
    let EditOperation::EditStructure {
        edit: StructureEdit::InsertApply { apply, .. },
    } = &proposal.batch.operations[2]
    else {
        panic!("expected apply insertion");
    };
    assert_eq!(apply.id.as_str(), "apl_provider_retouch");
    assert_eq!(
        apply.target,
        ApplyTarget::ItemSet {
            item_ids: vec![ItemId::new("itm_video").unwrap()]
        }
    );
    let ApplyOperation::Effect { effect } = &apply.stages[0].operation else {
        panic!("expected effect stage");
    };
    assert!(matches!(
        effect.parameters.get("radius"),
        Some(ParameterValue::NumberCurve { .. })
    ));
    let EditOperation::EditStructure {
        edit: StructureEdit::InsertRelation { relation },
    } = &proposal.batch.operations[3]
    else {
        panic!("expected matte relation");
    };
    assert!(
        matches!(&relation.kind, RelationKind::Matte { producer, consumer, .. }
        if producer.item_id().is_some() && consumer.apply_id() == Some(&apply.id))
    );
    assert!(matches!(
        &proposal.evidence[2],
        ProposalEvidence::RetouchApply { apply_id, .. } if apply_id == &apply.id
    ));
    assert!(matches!(
        apply_edit_batch(&project, &proposal.batch),
        EditOutcome::Applied { .. }
    ));
}

#[test]
fn retouch_preserves_control_to_stage_mapping() {
    let (project, request, response, context) = multi_control_fixture();
    let proposal = proposal_result(&project, &request, &response, &context).unwrap();
    let ProposalEvidence::RetouchApply { controls, .. } = &proposal.evidence[2] else {
        unreachable!()
    };
    assert_eq!(controls[0].control, "exposure");
    assert_eq!(controls[0].apply_stage_id.as_str(), "aps_provider_exposure");
    assert_eq!(controls[1].apply_stage_id.as_str(), "aps_provider_blur");
    assert!(canonical_edit_proposal_bytes(&proposal).is_ok());
}

#[test]
fn retouch_rejects_curve_tampering_after_rebind() {
    let (project, request, response, context) = retouch_fixture();
    let mut proposal = proposal_result(&project, &request, &response, &context).unwrap();
    let EditOperation::EditStructure {
        edit: StructureEdit::InsertApply { apply, .. },
    } = &mut proposal.batch.operations[2]
    else {
        unreachable!()
    };
    let ApplyOperation::Effect { effect } = &mut apply.stages[0].operation else {
        unreachable!()
    };
    effect
        .parameters
        .insert("radius".into(), ParameterValue::Number { value: 12.0 });
    let ProposalEvidence::RetouchApply { operation, .. } = &mut proposal.evidence[2] else {
        unreachable!()
    };
    *operation = OperationBinding::new(2, &proposal.batch.operations[2]).unwrap();
    assert!(canonical_edit_proposal_bytes(&proposal).is_err());
}

pub(super) fn retouch_fixture() -> (
    ProjectEnvelope,
    ProviderRequestEnvelope,
    ProviderResponseEnvelope,
    ApplicationContext,
) {
    let project = project();
    let (request, response) = exchange(Capability::Retouch);
    let ProviderOutput::Retouch(result) = &response.output else {
        unreachable!()
    };
    let context = retouch_context(result);
    (project, request, response, context)
}

pub(super) fn multi_control_fixture() -> (
    ProjectEnvelope,
    ProviderRequestEnvelope,
    ProviderResponseEnvelope,
    ApplicationContext,
) {
    let (project, mut request, _, _) = retouch_fixture();
    let controls = {
        let ProviderRequest::Retouch(value) = &mut request.request else {
            unreachable!()
        };
        let template = value.controls[0].clone();
        let mut exposure = template.clone();
        exposure.parameter = "exposure".into();
        let mut radius = template;
        radius.parameter = "radius".into();
        value.controls = vec![exposure, radius];
        value.controls.clone()
    };
    let mut output = crate::test_support::output_for(&request);
    let ProviderOutput::Retouch(result) = &mut output else {
        unreachable!()
    };
    result.controls = controls;
    let response = ProviderResponseEnvelope::new(&request, output).unwrap();
    let ProviderOutput::Retouch(result) = &response.output else {
        unreachable!()
    };
    let mut context = retouch_context(result);
    let application = retouch_mut(&mut context);
    application.effects = vec![
        effect_application("blur", "video.blur"),
        effect_application("exposure", "video.color_adjust"),
    ];
    application.controls = vec![
        control_application("exposure", "exposure", "brightness"),
        control_application("radius", "blur", "radius"),
    ];
    (project, request, response, context)
}

pub(super) fn proposal_result(
    project: &ProjectEnvelope,
    request: &ProviderRequestEnvelope,
    response: &ProviderResponseEnvelope,
    context: &ApplicationContext,
) -> ProviderResult<ProviderEditProposal> {
    propose_edit(project, request, response, context)
}

pub(super) fn retouch_mut(context: &mut ApplicationContext) -> &mut RetouchApplication {
    let ApplicationContext::Retouch(value) = context else {
        panic!("expected retouch context");
    };
    value
}

fn effect_application(name: &str, effect_type: &str) -> RetouchEffectApplication {
    RetouchEffectApplication {
        stage_id: ApplyStageId::new(format!("aps_provider_{name}")).unwrap(),
        active_range: None,
        effect: EffectInstance {
            id: EffectId::new(format!("fx_provider_{name}")).unwrap(),
            effect_type: effect_type.into(),
            enabled: true,
            enable_range: None,
            parameters: BTreeMap::new(),
        },
    }
}

fn control_application(control: &str, effect: &str, parameter: &str) -> RetouchControlApplication {
    RetouchControlApplication {
        control: control.into(),
        effect_id: EffectId::new(format!("fx_provider_{effect}")).unwrap(),
        effect_parameter: parameter.into(),
        keyframe_id_prefix: format!("kf_provider_{control}"),
    }
}
