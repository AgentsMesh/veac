use veac_ir::{Animatable, EditOperation, Interpolation, ItemId, Rect, VisualProperty};

use crate::*;

use super::support::*;

#[test]
fn dynamic_auto_reframe_maps_every_sample_to_one_canonical_crop_curve() {
    let (project, request, response, context) = dynamic_fixture();
    let proposal = propose_edit(&project, &request, &response, &context).unwrap();
    assert_eq!(proposal.batch.operations.len(), 1);
    assert_crop(&proposal.batch.operations[0]);
    assert!(matches!(
        &proposal.evidence[0],
        ProposalEvidence::CropSamples {
            operation,
            sample_indices,
            timebase: 100,
            keyframe_id_prefix,
            ..
        } if operation.operation_index == 0
            && sample_indices == &[0, 1]
            && keyframe_id_prefix == "kf_reframe"
    ));
    let veac_ir::EditOutcome::Applied {
        project: applied, ..
    } = veac_ir::apply_edit_batch(&project, &proposal.batch)
    else {
        panic!("dynamic reframe proposal must apply")
    };
    assert!(applied.project.sequences[0].tracks[3].clips[0]
        .visual
        .as_ref()
        .unwrap()
        .transform
        .crop
        .as_ref()
        .is_some_and(|crop| crop.keyframes().is_some()));
}

pub(super) fn dynamic_fixture() -> (
    veac_ir::ProjectEnvelope,
    ProviderRequestEnvelope,
    ProviderResponseEnvelope,
    ApplicationContext,
) {
    let (mut request, mut response) = exchange(Capability::AutoReframe);
    let ProviderRequest::AutoReframe(value) = &mut request.request else {
        unreachable!()
    };
    value.target_aspect_ratio = veac_ir::Rational::new(16, 9).unwrap();
    let ProviderOutput::AutoReframe(output) = &mut response.output else {
        unreachable!()
    };
    output.crops = vec![sample(10, 0.1, 0.1), sample(11, 0.375, 0.25)];
    response = ProviderResponseEnvelope::new(&request, response.output).unwrap();
    let mut context = reframe_context();
    let ApplicationContext::AutoReframe(value) = &mut context else {
        unreachable!()
    };
    value.clip_id = ItemId::new("itm_video").unwrap();
    value.time.provider_origin = time(10);
    value.time.clip_local_origin = time(20);
    (project(), request, response, context)
}

fn sample(time_value: i64, x: f64, y: f64) -> CropSample {
    CropSample {
        time: time(time_value),
        rect: Rect {
            x,
            y,
            width: 0.5,
            height: 0.5,
        },
        confidence: 0.9,
    }
}

fn assert_crop(operation: &EditOperation) {
    let EditOperation::SetVisualProperty {
        property: VisualProperty::Crop(Some(Animatable::Keyframes { keyframes })),
        ..
    } = operation
    else {
        panic!("crop curve")
    };
    assert_eq!(keyframes.len(), 2);
    assert_eq!(keyframes[0].time, time(20));
    assert_eq!(keyframes[1].time, time(21));
    assert_eq!(keyframes[0].id.as_str(), "kf_reframe_crop_0001");
    assert_eq!(keyframes[1].id.as_str(), "kf_reframe_crop_0002");
    assert!(keyframes
        .iter()
        .all(|key| matches!(key.interpolation, Interpolation::Linear)));
}
