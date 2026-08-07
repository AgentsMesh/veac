use std::collections::BTreeSet;

use veac_plan::{
    canonical::*, plan_hash, resolve, validate_render_plan, CURRENT_RENDER_PLAN_VERSION,
};

use crate::fixtures::project_with_clips;

#[test]
fn plan_v6_keeps_only_the_selected_temporal_closure_and_source_identity() {
    let enabled = [true, false, true, false, true];
    let (project, ids) = project_with_clips(&enabled);
    let plan = resolve(&project, None).unwrap().remove(0);
    assert_eq!(plan.header.schema_version, CURRENT_RENDER_PLAN_VERSION);
    assert_eq!(CURRENT_RENDER_PLAN_VERSION, 6);
    assert_eq!(plan.header.source.executable, project.executable);
    assert_eq!(
        plan.header.cache.project_semantic_sha256,
        plan.header.source.semantic_hash
    );
    assert_eq!(
        plan.header.cache.project_snapshot_sha256,
        plan.header.source.snapshot_hash
    );
    for digest in [
        &plan.header.cache.executable_manifest_sha256,
        &plan.header.cache.temporal_library_sha256,
        &plan.header.cache.resolver_sha256,
    ] {
        assert_eq!(digest.len(), 64);
    }
    let actual = plan
        .temporal
        .bindings
        .iter()
        .map(|value| value.id.clone())
        .collect::<Vec<_>>();
    let expected = ids
        .into_iter()
        .zip(enabled)
        .filter_map(|(id, enabled)| enabled.then_some(id))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
    assert_eq!(plan.temporal.programs.len(), 1);
    assert_eq!(plan.temporal.provenance.len(), 1);
    assert_eq!(plan.temporal.programs[0].inputs.len(), 1);
    let (binding, program) = plan.temporal_program_for(&actual[0]).unwrap();
    assert_eq!(binding.program_id, program.id);
    assert_eq!(plan.temporal_binding(&actual[0]), Some(binding));
    assert!(plan
        .temporal_program_for(&TemporalBindingId::new("tbd_absent").unwrap())
        .is_none());
    validate_render_plan(&plan).unwrap();
    assert_eq!(plan_hash(&plan).unwrap(), plan_hash(&plan.clone()).unwrap());
}

#[test]
fn fully_pruned_temporal_content_produces_an_empty_closed_library() {
    let (project, _) = project_with_clips(&[false, false, false]);
    let plan = resolve(&project, None).unwrap().remove(0);
    assert!(plan.temporal.programs.is_empty());
    assert!(plan.temporal.bindings.is_empty());
    assert!(plan.temporal.provenance.is_empty());
    assert_eq!(plan.temporal.opset_version, TEMPORAL_OPSET_VERSION);
    validate_render_plan(&plan).unwrap();
}

#[test]
fn deterministic_random_reachability_matches_every_enabled_sink() {
    let mut state = 0x9e37_79b9_u64;
    for _case in 0..64 {
        let enabled = (0..24)
            .map(|_| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                state >> 63 == 1
            })
            .collect::<Vec<_>>();
        let (project, ids) = project_with_clips(&enabled);
        let plan = resolve(&project, None).unwrap().remove(0);
        let actual = plan
            .temporal
            .bindings
            .iter()
            .map(|value| value.id.clone())
            .collect::<BTreeSet<_>>();
        let expected = ids
            .into_iter()
            .zip(enabled)
            .filter_map(|(id, enabled)| enabled.then_some(id))
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected);
        assert_eq!(plan.temporal.programs.is_empty(), actual.is_empty());
        assert_eq!(plan.temporal.provenance.is_empty(), actual.is_empty());
        validate_render_plan(&plan).unwrap();
    }
}
