use veac_ir::{Animatable, EditOperation, Length, LengthUnit, Point, RationalTime, VisualProperty};

use crate::*;

use super::support::*;

#[test]
fn tracking_maps_explicit_units_and_exact_clip_local_times() {
    let project = project();
    let (request, mut response) = exchange(Capability::MotionTracking);
    let ProviderOutput::MotionTracking(result) = &mut response.output else {
        unreachable!()
    };
    result.transforms[0].time = time(10);
    let mut second = result.transforms[0].clone();
    second.time = time(11);
    second.position = Point {
        x: Length {
            value: 320.0,
            unit: LengthUnit::Pixels,
        },
        y: Length {
            value: 25.0,
            unit: LengthUnit::Percent,
        },
    };
    result.transforms.push(second);
    let mut context = transform_context(Capability::MotionTracking);
    let ApplicationContext::MotionTracking(value) = &mut context else {
        unreachable!()
    };
    value.time.provider_origin = time(10);
    value.time.clip_local_origin = time(20);
    let proposal = propose_edit(&project, &request, &response, &context).unwrap();
    assert_eq!(proposal.batch.operations.len(), 3);
    let EditOperation::SetVisualProperty {
        property: VisualProperty::Position(Animatable::Keyframes { keyframes }),
        ..
    } = &proposal.batch.operations[0]
    else {
        unreachable!()
    };
    assert_eq!(keyframes[0].time, time(20));
    assert_eq!(keyframes[1].time, time(21));
    assert_eq!(keyframes[1].value.x.unit, LengthUnit::Pixels);
    assert_eq!(keyframes[1].value.y.unit, LengthUnit::Percent);
    assert!(matches!(
        &proposal.evidence[2],
        ProposalEvidence::TransformSamples { operation, sample_indices, component }
            if operation.operation_index == 2
                && sample_indices == &[0, 1]
                && *component == TransformComponent::Rotation
    ));
}

#[test]
fn tracking_rejects_inexact_out_of_range_or_unordered_samples() {
    let (request, response) = exchange(Capability::MotionTracking);
    let mut inexact = response.clone();
    let ProviderOutput::MotionTracking(result) = &mut inexact.output else {
        unreachable!()
    };
    result.transforms[0].time = RationalTime::new(1, 3).unwrap();
    assert_invalid(propose_edit(
        &project(),
        &request,
        &inexact,
        &transform_context(Capability::MotionTracking),
    ));

    let mut outside = response.clone();
    let ProviderOutput::MotionTracking(result) = &mut outside.output else {
        unreachable!()
    };
    result.transforms[0].time = time(101);
    assert_invalid(propose_edit(
        &project(),
        &request,
        &outside,
        &transform_context(Capability::MotionTracking),
    ));

    let mut unordered = response;
    let ProviderOutput::MotionTracking(result) = &mut unordered.output else {
        unreachable!()
    };
    result.transforms[0].time = time(1);
    result.transforms.push(result.transforms[0].clone());
    assert_invalid(propose_edit(
        &project(),
        &request,
        &unordered,
        &transform_context(Capability::MotionTracking),
    ));
}

#[test]
fn tracking_rejects_locked_targets_bad_ids_and_empty_output() {
    let (request, response) = exchange(Capability::MotionTracking);
    let mut value = project();
    value.project.sequences[0].tracks[0].state.locked = true;
    assert_invalid(propose_edit(
        &value,
        &request,
        &response,
        &transform_context(Capability::MotionTracking),
    ));

    let mut context = transform_context(Capability::MotionTracking);
    let ApplicationContext::MotionTracking(value) = &mut context else {
        unreachable!()
    };
    value.keyframe_id_prefix = "bad".into();
    assert_invalid(propose_edit(&project(), &request, &response, &context));

    let (request, mut response) = exchange(Capability::MotionTracking);
    let ProviderOutput::MotionTracking(result) = &mut response.output else {
        unreachable!()
    };
    result.transforms.clear();
    assert_unsupported(propose_edit(
        &project(),
        &request,
        &response,
        &transform_context(Capability::MotionTracking),
    ));
}

fn assert_invalid(value: ProviderResult<ProviderEditProposal>) {
    assert_eq!(value.unwrap_err().kind, ProviderErrorKind::InvalidContract);
}

fn assert_unsupported(value: ProviderResult<ProviderEditProposal>) {
    assert_eq!(
        value.unwrap_err().kind,
        ProviderErrorKind::UnsupportedApplication
    );
}
