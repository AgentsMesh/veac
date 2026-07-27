use std::collections::BTreeMap;

use veac_ir::*;

use crate::*;

use super::retouch_tests::{proposal_result, retouch_fixture, retouch_mut};
use super::support::{retouch_context, time};

#[test]
fn retouch_without_artifact_builds_direct_apply() {
    let (project, request, response, _) = retouch_fixture();
    let mut output = response.output.clone();
    let ProviderOutput::Retouch(result) = &mut output else {
        unreachable!()
    };
    result.masks.clear();
    let context = retouch_context(result);
    let response = ProviderResponseEnvelope::new(&request, output).unwrap();
    let proposal = proposal_result(&project, &request, &response, &context).unwrap();
    assert_eq!(proposal.batch.operations.len(), 1);
    assert!(matches!(
        proposal.batch.operations[0],
        EditOperation::EditStructure {
            edit: StructureEdit::InsertApply { .. }
        }
    ));
    assert!(matches!(
        apply_edit_batch(&project, &proposal.batch),
        EditOutcome::Applied { .. }
    ));
}

#[test]
fn retouch_preserves_apply_anchor() {
    let (mut project, request, response, mut context) = retouch_fixture();
    project.project.sequences[0].applies.push(anchor_apply());
    project = ProjectEnvelope::new(project.project);
    retouch_mut(&mut context).before_apply_id = Some(ApplyId::new("apl_anchor").unwrap());
    let proposal = proposal_result(&project, &request, &response, &context).unwrap();
    assert!(
        matches!(&proposal.batch.operations[2], EditOperation::EditStructure {
        edit: StructureEdit::InsertApply { before_id: Some(value), after_id: None, .. }
    } if value.as_str() == "apl_anchor")
    );
    let EditOutcome::Applied {
        project: edited, ..
    } = apply_edit_batch(&project, &proposal.batch)
    else {
        unreachable!()
    };
    assert_eq!(
        edited.project.sequences[0].applies[0].id.as_str(),
        "apl_provider_retouch"
    );
}

fn anchor_apply() -> Apply {
    Apply {
        id: ApplyId::new("apl_anchor").unwrap(),
        enabled: true,
        record_range: TimeRange::new(time(0), time(100)).unwrap(),
        target: ApplyTarget::ItemSet {
            item_ids: vec![ItemId::new("itm_video").unwrap()],
        },
        stages: vec![ApplyStage {
            id: ApplyStageId::new("aps_anchor").unwrap(),
            enabled: true,
            active_range: None,
            operation: ApplyOperation::Effect {
                effect: EffectInstance {
                    id: EffectId::new("fx_anchor").unwrap(),
                    effect_type: "video.blur".into(),
                    enabled: true,
                    enable_range: None,
                    parameters: BTreeMap::from([(
                        "radius".into(),
                        ParameterValue::Number { value: 1.0 },
                    )]),
                },
            },
        }],
        mix: ApplyMix::default(),
    }
}
