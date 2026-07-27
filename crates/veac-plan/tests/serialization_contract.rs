mod support;

use std::error::Error;

use support::project;
use veac_plan::{
    canonical::Animatable, canonical_plan_bytes, canonical_plan_json, plan_hash,
    render_plan_json_schema, resolve, PlanInputId, PlanOutputId, ResolvedRenderPlan,
};

#[test]
fn plan_has_stable_jcs_hash_schema_and_serde_round_trip() {
    let plan = resolve(&project(), None).unwrap().remove(0);
    let json = canonical_plan_json(&plan).unwrap();
    let bytes = canonical_plan_bytes(&plan).unwrap();
    assert_eq!(json.as_bytes(), bytes);
    assert_eq!(plan_hash(&plan).unwrap(), plan_hash(&plan.clone()).unwrap());
    assert_eq!(plan_hash(&plan).unwrap().len(), 64);
    let decoded: ResolvedRenderPlan = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, plan);
    let schema = render_plan_json_schema().unwrap();
    assert_eq!(schema["title"], "ResolvedRenderPlan");
    assert!(schema["properties"]["inputs"].is_object());
}

#[test]
fn machine_local_execution_bindings_are_absent_from_plan_json() {
    let plan = resolve(&project(), None).unwrap().remove(0);
    let value = serde_json::to_value(&plan).unwrap();
    assert!(value.get("bindings").is_none());
    assert!(value["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .all(|input| input.get("path").is_none()));
}

#[test]
fn non_finite_plan_values_return_serialization_errors() {
    let mut plan = resolve(&project(), None).unwrap().remove(0);
    plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .opacity = Animatable::constant(f64::NAN);
    let error = canonical_plan_json(&plan).unwrap_err();
    assert!(error.to_string().contains("serialization failed"));
    assert!(error.source().is_some());
    assert!(canonical_plan_bytes(&plan).is_err());
    assert!(plan_hash(&plan).is_err());
}

#[test]
fn plan_ids_are_typed_and_support_maximum_canonical_suffixes() {
    let input = PlanInputId::new(format!("pin_{}", "a".repeat(124))).unwrap();
    let output = PlanOutputId::new(format!("pout_{}", "b".repeat(124))).unwrap();
    assert_eq!(input.as_str().len(), 128);
    assert_eq!(output.as_str().len(), 129);
    assert_eq!(output.to_string(), output.as_str());
    let error = PlanInputId::new("bad").unwrap_err();
    assert!(error.to_string().contains("expected prefix"));
    assert!(PlanInputId::new("pin_bad/slash").is_err());
    assert!(PlanOutputId::new(format!("pout_{}", "x".repeat(125))).is_err());
}
