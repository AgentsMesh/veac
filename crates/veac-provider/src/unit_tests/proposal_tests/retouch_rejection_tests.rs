use std::collections::BTreeMap;

use veac_ir::TimeRange;

use veac_ir::{EffectId, EffectInstance, ParameterValue};

use crate::*;

use super::retouch_tests::multi_control_fixture;
use super::support::*;

#[test]
fn retouch_rejects_outputs_without_one_executable_matte() {
    let (request, mut response) = exchange(Capability::Retouch);
    let context = match &response.output {
        ProviderOutput::Retouch(result) => retouch_context(result),
        _ => unreachable!(),
    };
    let ProviderOutput::Retouch(result) = &mut response.output else {
        unreachable!()
    };
    result.masks.push(result.masks[0].clone());
    assert_unsupported(propose_edit(&project(), &request, &response, &context));
}

#[test]
fn retouch_rejects_noncanonical_or_unexecutable_effect_bindings() {
    let (project, request, response, mut missing) = multi_control_fixture();
    retouch_mut(&mut missing).effects.pop();
    assert_invalid(propose_edit(&project, &request, &response, &missing));

    let (project, request, response, mut unknown) = multi_control_fixture();
    retouch_mut(&mut unknown).effects[0].effect.effect_type = "vendor.face_magic".into();
    assert_unsupported(propose_edit(&project, &request, &response, &unknown));

    let (project, request, response, mut prefilled) = multi_control_fixture();
    retouch_mut(&mut prefilled).effects[0]
        .effect
        .parameters
        .insert("radius".into(), ParameterValue::Number { value: 0.5 });
    assert_invalid(propose_edit(&project, &request, &response, &prefilled));
}

#[test]
fn retouch_rejects_duplicate_targets_and_out_of_range_samples() {
    let (project, request, response, mut duplicate) = multi_control_fixture();
    let value = retouch_mut(&mut duplicate);
    value.controls[1].effect_id = value.controls[0].effect_id.clone();
    value.controls[1].effect_parameter = value.controls[0].effect_parameter.clone();
    assert_invalid(propose_edit(&project, &request, &response, &duplicate));

    let (project, request, mut response, context) = multi_control_fixture();
    let ProviderOutput::Retouch(result) = &mut response.output else {
        unreachable!()
    };
    result.controls[0].samples[0].time = time(101);
    assert_invalid(propose_edit(&project, &request, &response, &context));
}

#[test]
fn retouch_trial_apply_rejects_effect_ids_owned_elsewhere_in_the_project() {
    let (mut project, request, response, context) = multi_control_fixture();
    project.project.sequences[0].tracks[3].clips[0]
        .effects
        .push(EffectInstance {
            id: EffectId::new("fx_provider_blur").unwrap(),
            effect_type: "video.blur".into(),
            enabled: true,
            enable_range: None,
            parameters: BTreeMap::from([("radius".into(), ParameterValue::Number { value: 1.0 })]),
        });
    assert_invalid(propose_edit(&project, &request, &response, &context));
}

#[test]
fn retouch_rejects_static_parameter_mappings_and_values_outside_effect_ranges() {
    let (canonical, request, response, mut static_target) = multi_control_fixture();
    let value = retouch_mut(&mut static_target);
    value.effects[1].effect.effect_type = "video.stabilize".into();
    value.controls[1].effect_parameter = "enabled".into();
    assert_unsupported(propose_edit(
        &canonical,
        &request,
        &response,
        &static_target,
    ));

    let (mut request, _) = exchange(Capability::Retouch);
    let ProviderRequest::Retouch(value) = &mut request.request else {
        unreachable!()
    };
    value.controls[0].samples[0].value = 101.0;
    let controls = value.controls.clone();
    let mut output = crate::test_support::output_for(&request);
    let ProviderOutput::Retouch(result) = &mut output else {
        unreachable!()
    };
    result.controls = controls;
    let response = ProviderResponseEnvelope::new(&request, output).unwrap();
    let ProviderOutput::Retouch(result) = &response.output else {
        unreachable!()
    };
    assert_unsupported(propose_edit(
        &project(),
        &request,
        &response,
        &retouch_context(result),
    ));
}

#[test]
fn retouch_rejects_mismatched_results_unsorted_bindings_and_invalid_key_prefixes() {
    let (request, mut response) = exchange(Capability::Retouch);
    let context = retouch_context(result(&response));
    let ProviderOutput::Retouch(result) = &mut response.output else {
        unreachable!()
    };
    result.controls[0].samples[0].value = 0.6;
    assert_invalid(propose_edit(&project(), &request, &response, &context));

    let (project, request, response, mut unsorted) = multi_control_fixture();
    retouch_mut(&mut unsorted).controls.reverse();
    assert_invalid(propose_edit(&project, &request, &response, &unsorted));

    let (project, request, response, mut prefix) = multi_control_fixture();
    retouch_mut(&mut prefix).controls[0].keyframe_id_prefix = "invalid prefix".into();
    assert_invalid(propose_edit(&project, &request, &response, &prefix));

    let (project, request, response, mut range) = multi_control_fixture();
    retouch_mut(&mut range).record_range = TimeRange::new(time(1), time(99)).unwrap();
    assert_invalid(propose_edit(&project, &request, &response, &range));
}

fn result(response: &ProviderResponseEnvelope) -> &RetouchResult {
    let ProviderOutput::Retouch(result) = &response.output else {
        unreachable!()
    };
    result
}

fn retouch_mut(value: &mut ApplicationContext) -> &mut RetouchApplication {
    let ApplicationContext::Retouch(value) = value else {
        unreachable!()
    };
    value
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
