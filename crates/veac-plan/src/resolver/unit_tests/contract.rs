use std::error::Error;

use super::support::*;
use crate::{canonical::*, *};

#[test]
fn public_plan_serialization_and_binding_contract() {
    let plan = resolve(&project(), None).unwrap().remove(0);
    assert_eq!(
        plan.inputs[0]
            .video
            .as_ref()
            .unwrap()
            .selection
            .global_index,
        0
    );
    assert!(plan.sequences[0].tracks[0].clips[0].visual.is_some());
    assert_eq!(plan.sequences[0].duration, time(600));
    let json = canonical_plan_json(&plan).unwrap();
    assert_eq!(json.as_bytes(), canonical_plan_bytes(&plan).unwrap());
    assert_eq!(plan_hash(&plan).unwrap().len(), 64);
    assert_eq!(
        serde_json::from_str::<ResolvedRenderPlan>(&json).unwrap(),
        plan
    );
    assert!(render_plan_json_schema().unwrap()["properties"].is_object());

    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(value.get("bindings").is_none());
    assert!(value["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .all(|input| input.get("path").is_none()));

    let mut invalid = plan;
    invalid.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .opacity = Animatable::constant(f64::NAN);
    let error = canonical_plan_json(&invalid).unwrap_err();
    assert!(error
        .to_string()
        .contains("render plan serialization failed"));
    assert!(error.source().is_some());
    assert!(canonical_plan_bytes(&invalid).is_err());
    assert!(plan_hash(&invalid).is_err());
}

#[test]
fn public_ids_and_resolution_error_accessors() {
    let input = PlanInputId::new("pin_video").unwrap();
    let output = PlanOutputId::new("pout_main").unwrap();
    assert_eq!(input.as_str(), "pin_video");
    assert_eq!(output.to_string(), "pout_main");
    assert!(PlanInputId::new("bad")
        .unwrap_err()
        .to_string()
        .contains("prefix"));
    assert!(PlanOutputId::new("pout_bad/slash").is_err());

    let missing = RenderConfigId::new("out_missing").unwrap();
    let error = resolve_one(&project(), &missing).unwrap_err();
    assert!(error
        .to_string()
        .contains("1 resolution error(s); first is RENDER_CONFIG_NOT_FOUND"));
    assert_eq!(
        error.diagnostics()[0].kind,
        ResolutionErrorKind::RenderConfigNotFound
    );
    assert_eq!(error.into_diagnostics().len(), 1);
    assert_eq!(
        ResolutionErrors::new(Vec::new()).to_string(),
        "resolution failed without a diagnostic"
    );
}

#[test]
fn resolving_all_configs_returns_stable_id_order() {
    let mut value = project();
    let mut alpha = value.project.render_configs[0].clone();
    alpha.id = RenderConfigId::new("out_alpha").unwrap();
    alpha.deliverables[0].id = DeliverableId::new("dlv_alpha").unwrap();
    alpha.deliverables[0].file_name = "alpha.mp4".to_owned();
    value.project.render_configs[0].id = RenderConfigId::new("out_zulu").unwrap();
    value.project.render_configs.push(alpha);

    let plans = resolve(&value, None).unwrap();
    let ids: Vec<_> = plans
        .iter()
        .map(|plan| plan.output.render_config_id.as_str())
        .collect();
    assert_eq!(ids, ["out_alpha", "out_zulu"]);
}

#[test]
fn media_resolution_error_matrix() {
    let mut value = project();
    value.project.materials[0].identity = None;
    assert_code(value, "MATERIAL_IDENTITY_MISSING");

    let mut value = project();
    value.project.materials[0].probe = None;
    assert_code(value, "MATERIAL_PROBE_MISSING");

    let mut value = project();
    value.project.materials[0] = remote_material("med_video");
    assert_code(value, "REMOTE_MATERIAL_UNRESOLVED");

    let mut value = project();
    let probe = value.project.materials[0].probe.as_mut().unwrap();
    probe.container_duration = None;
    probe.streams[0].duration = None;
    assert_code(value, "SOURCE_DURATION_UNAVAILABLE");

    let mut value = project();
    let probe = value.project.materials[0].probe.as_mut().unwrap();
    probe.container_duration = Some(time(300));
    probe.streams[0].duration = Some(time(300));
    assert_code(value, "SOURCE_RANGE_OUT_OF_BOUNDS");
}

#[test]
fn canonical_probe_failures_keep_resolution_kinds() {
    let mut value = project();
    value.project.materials[0].stream_intent.video = StreamChoice::GlobalIndex { global_index: 9 };
    let error = resolve(&value, None).unwrap_err();
    assert_eq!(
        error.diagnostics()[0].kind,
        ResolutionErrorKind::StreamSelectionMismatch
    );

    let mut value = project();
    value.project.materials[0].identity = Some(identity('c'));
    let error = resolve(&value, None).unwrap_err();
    assert_eq!(
        error.diagnostics()[0].kind,
        ResolutionErrorKind::MaterialProbeIdentityMismatch
    );
}

fn assert_code(project: ProjectEnvelope, code: &str) {
    let error = resolve(&project, None).unwrap_err();
    assert!(error.diagnostics().iter().any(|item| item.code == code));
}
