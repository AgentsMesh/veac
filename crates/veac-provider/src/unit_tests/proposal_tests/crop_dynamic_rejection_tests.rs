use veac_artifact::ContentDigest;
use veac_ir::{Animatable, FitMode, Frame, Length, LengthUnit, Rational, Rect};

use crate::*;

use super::crop_dynamic_tests::dynamic_fixture;

#[test]
fn dynamic_reframe_does_not_require_the_request_ratio_to_equal_the_sequence() {
    let (project, mut request, response, context) = dynamic_fixture();
    let ProviderRequest::AutoReframe(value) = &mut request.request else {
        unreachable!()
    };
    value.target_aspect_ratio = Rational::new(9, 16).unwrap();
    let response = ProviderResponseEnvelope::new(&request, response.output).unwrap();
    let proposal = propose_edit(&project, &request, &response, &context).unwrap();
    assert!(matches!(
        veac_ir::apply_edit_batch(&project, &proposal.batch),
        veac_ir::EditOutcome::Applied { .. }
    ));
}

#[test]
fn dynamic_reframe_composes_with_authored_placement_frame_and_crop_state() {
    let (mut project, request, response, context) = dynamic_fixture();
    let visual = visual(&mut project);
    visual.frame = Some(Frame {
        width: pixels(400.0),
        height: pixels(700.0),
        fit: FitMode::Cover,
    });
    visual.transform.crop = Some(Animatable::constant(Rect {
        x: 0.0,
        y: 0.0,
        width: 0.8,
        height: 0.8,
    }));
    let proposal = propose_edit(&project, &request, &response, &context).unwrap();
    let veac_ir::EditOutcome::Applied {
        project: applied, ..
    } = veac_ir::apply_edit_batch(&project, &proposal.batch)
    else {
        panic!("reframe must apply with authored frame state")
    };
    let visual = applied.project.sequences[0].tracks[3].clips[0]
        .visual
        .as_ref()
        .unwrap();
    assert_eq!(visual.frame.unwrap().width.value, 400.0);
    assert!(visual
        .transform
        .crop
        .as_ref()
        .unwrap()
        .keyframes()
        .is_some());
}

#[test]
fn dynamic_reframe_rejects_request_input_and_stream_mismatches() {
    let (project, mut request, response, context) = dynamic_fixture();
    let ProviderRequest::AutoReframe(value) = &mut request.request else {
        unreachable!()
    };
    value.video.content = ContentDigest::sha256(b"different-video");
    let response = ProviderResponseEnvelope::new(&request, response.output).unwrap();
    assert_invalid(propose_edit(&project, &request, &response, &context));

    let (project, mut request, response, context) = dynamic_fixture();
    let ProviderRequest::AutoReframe(value) = &mut request.request else {
        unreachable!()
    };
    value.video.stream_index = Some(1);
    let response = ProviderResponseEnvelope::new(&request, response.output).unwrap();
    assert_invalid(propose_edit(&project, &request, &response, &context));
}

#[test]
fn dynamic_reframe_rejects_invalid_key_ids_and_clip_local_time() {
    let (project, request, response, mut context) = dynamic_fixture();
    if let ApplicationContext::AutoReframe(value) = &mut context {
        value.keyframe_id_prefix = "invalid prefix".into();
    }
    assert_invalid(propose_edit(&project, &request, &response, &context));

    if let ApplicationContext::AutoReframe(value) = &mut context {
        value.keyframe_id_prefix = "kf_reframe".into();
        value.time.clip_local_origin = veac_ir::RationalTime::new(-11, 100).unwrap();
    }
    assert_invalid(propose_edit(&project, &request, &response, &context));
}

fn visual(project: &mut veac_ir::ProjectEnvelope) -> &mut veac_ir::VisualProperties {
    project.project.sequences[0].tracks[3].clips[0]
        .visual
        .as_mut()
        .unwrap()
}

fn pixels(value: f64) -> Length {
    Length {
        value,
        unit: LengthUnit::Pixels,
    }
}

fn assert_invalid(value: ProviderResult<ProviderEditProposal>) {
    assert_eq!(value.unwrap_err().kind, ProviderErrorKind::InvalidContract);
}
