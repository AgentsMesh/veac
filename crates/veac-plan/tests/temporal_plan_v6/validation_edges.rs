use veac_plan::{canonical::*, validate_render_plan};

use crate::validation::{codes, plan};

#[test]
fn missing_program_or_clock_owner_and_independent_type_mismatch_fail_closed() {
    let mut missing_program = plan(1);
    missing_program.temporal.programs.clear();
    let actual = codes(&missing_program);
    assert!(actual.contains(&"TEMPORAL_PROGRAM_MISSING".to_owned()));
    assert!(!actual.contains(&"PLAN_TEMPORAL_PROGRAM_ORPHAN".to_owned()));
    assert!(missing_program
        .temporal_program_for(&missing_program.temporal.bindings[0].id)
        .is_none());

    let mut missing_owner = plan(1);
    missing_owner.temporal.bindings[0].clocks[0].owner = TemporalClockOwner::Item {
        item_id: ItemId::new("itm_not_in_plan").unwrap(),
    };
    let actual = codes(&missing_owner);
    assert!(actual.contains(&"PLAN_TEMPORAL_CLOCK_OWNER_MISSING".to_owned()));
    assert!(actual.contains(&"PLAN_TEMPORAL_CROSS_OWNER".to_owned()));

    let mut result_type = plan(1);
    result_type.temporal.programs[0].result_type = TemporalType::Vec2;
    let actual = codes(&result_type);
    assert!(actual.contains(&"TEMPORAL_PROGRAM_RESULT".to_owned()));
    assert!(actual.contains(&"TEMPORAL_BINDING_RESULT".to_owned()));
    assert!(actual.contains(&"PLAN_TEMPORAL_PROGRAM_TYPE".to_owned()));
}

#[test]
fn duplicate_program_provenance_and_orphan_binding_fail_closed() {
    let mut duplicates = plan(1);
    duplicates
        .temporal
        .programs
        .push(duplicates.temporal.programs[0].clone());
    duplicates
        .temporal
        .provenance
        .push(duplicates.temporal.provenance[0].clone());
    let actual = codes(&duplicates);
    assert!(actual.contains(&"TEMPORAL_PROGRAM_DUPLICATE".to_owned()));
    assert!(actual.contains(&"TEMPORAL_PROVENANCE_DUPLICATE".to_owned()));
    assert!(actual.contains(&"PLAN_TEMPORAL_ORDER".to_owned()));

    let mut orphan = plan(1);
    let mut binding = orphan.temporal.bindings[0].clone();
    binding.id = TemporalBindingId::new("tbd_unused").unwrap();
    orphan.temporal.bindings.push(binding);
    assert!(codes(&orphan).contains(&"PLAN_TEMPORAL_BINDING_ORPHAN".to_owned()));
}

#[test]
fn malformed_source_manifest_resolver_cache_and_graph_fields_fail_closed() {
    let mut value = plan(1);
    value.header.source.project_id = serde_json::from_str("\"bad\"").unwrap();
    value.header.source.revision = MAX_SAFE_INTEGER + 1;
    value.header.source.timebase = 0;
    value.header.source.snapshot_hash = "A".repeat(64);
    value.header.source.executable.language_version = "01.2.3".to_owned();
    value.header.source.executable.digests.compiler_sha256 = "not-a-digest".to_owned();
    value.header.resolver.stream_selection_policy = "".to_owned();
    value.header.cache.resolver_sha256 = "BAD".to_owned();
    value.entry_sequence_id = SequenceId::new("seq_absent").unwrap();
    value.inputs.push(value.inputs[0].clone());
    let errors = validate_render_plan(&value).unwrap_err();
    assert!(!errors.diagnostics().is_empty());
    let actual = errors
        .into_diagnostics()
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    for expected in [
        "PLAN_SOURCE",
        "PLAN_SOURCE_DIGEST",
        "PLAN_EXECUTABLE_MANIFEST",
        "PLAN_EXECUTABLE_DIGEST",
        "PLAN_RESOLVER_IDENTITY",
        "PLAN_CACHE_DIGEST",
        "PLAN_CACHE_IDENTITY",
        "PLAN_OUTPUT_SEQUENCE",
        "PLAN_ENTRY_SEQUENCE",
        "PLAN_INPUT_ORDER",
    ] {
        assert!(actual.contains(&expected.to_owned()), "missing {expected}");
    }
}
