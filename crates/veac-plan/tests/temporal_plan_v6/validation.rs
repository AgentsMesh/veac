use std::error::Error;

use veac_plan::{
    canonical::*, canonical_plan_json, decode_render_plan_json, resolve, validate_render_plan,
    PlanSerializationError, ResolvedRenderPlan,
};

use crate::fixtures::{binding, literal_program, project_with_clips, provenance};

pub(super) fn plan(count: usize) -> ResolvedRenderPlan {
    let enabled = vec![true; count];
    resolve(&project_with_clips(&enabled).0, None)
        .unwrap()
        .remove(0)
}

pub(super) fn codes(plan: &ResolvedRenderPlan) -> Vec<String> {
    validate_render_plan(plan)
        .unwrap_err()
        .into_diagnostics()
        .into_iter()
        .map(|value| value.code)
        .collect()
}

fn temporal_opacity(plan: &mut ResolvedRenderPlan) -> &mut Animatable<f64> {
    &mut plan
        .sequences
        .iter_mut()
        .flat_map(|sequence| &mut sequence.tracks)
        .find(|track| track.id.as_str() == "trk_temporal")
        .unwrap()
        .clips[0]
        .visual
        .as_mut()
        .unwrap()
        .opacity
}

#[test]
fn strict_decoder_rejects_unknown_duplicate_and_invalid_plan_data() {
    let plan = plan(1);
    let json = canonical_plan_json(&plan).unwrap();
    assert_eq!(decode_render_plan_json(&json).unwrap(), plan);

    let mut unknown = serde_json::to_value(&plan).unwrap();
    unknown["header"]["unknown"] = true.into();
    assert!(matches!(
        decode_render_plan_json(&unknown.to_string()),
        Err(PlanSerializationError::Json(_))
    ));
    let duplicate = json.replacen(
        "\"schema\":\"https://veac.dev/schemas/render-plan\"",
        concat!(
            "\"schema\":\"https://veac.dev/schemas/render-plan\",",
            "\"schema\":\"https://veac.dev/schemas/render-plan\""
        ),
        1,
    );
    assert!(matches!(
        decode_render_plan_json(&duplicate),
        Err(PlanSerializationError::Json(_))
    ));

    let mut invalid = serde_json::to_value(plan).unwrap();
    invalid["header"]["schema_version"] = 5.into();
    let error = decode_render_plan_json(&invalid.to_string()).unwrap_err();
    assert!(matches!(error, PlanSerializationError::Validation(_)));
    assert!(error.to_string().contains("validation failed"));
    assert!(error.source().is_some());
}

#[test]
fn missing_mismatched_and_cross_owner_bindings_fail_closed() {
    let mut missing = plan(1);
    *temporal_opacity(&mut missing) = binding(&TemporalBindingId::new("tbd_unknown").unwrap());
    let actual = codes(&missing);
    assert!(actual.contains(&"PLAN_TEMPORAL_BINDING_MISSING".to_owned()));
    assert!(actual.contains(&"PLAN_TEMPORAL_BINDING_ORPHAN".to_owned()));

    let mut mismatch = plan(1);
    mismatch.temporal.bindings[0].result_type = TemporalType::Vec2;
    let actual = codes(&mismatch);
    assert!(actual.contains(&"PLAN_TEMPORAL_SINK_TYPE".to_owned()));
    assert!(actual.contains(&"TEMPORAL_BINDING_RESULT".to_owned()));

    let mut owner = plan(1);
    owner.temporal.bindings[0].clocks[0].owner = TemporalClockOwner::Item {
        item_id: ItemId::new("itm_video").unwrap(),
    };
    assert!(codes(&owner).contains(&"PLAN_TEMPORAL_CROSS_OWNER".to_owned()));
}

#[test]
fn duplicate_unordered_and_orphan_temporal_records_fail_closed() {
    let mut duplicate = plan(2);
    duplicate
        .temporal
        .bindings
        .push(duplicate.temporal.bindings[0].clone());
    let actual = codes(&duplicate);
    assert!(actual.contains(&"TEMPORAL_BINDING_DUPLICATE".to_owned()));
    assert!(actual.contains(&"PLAN_TEMPORAL_ORDER".to_owned()));

    let mut unordered = plan(2);
    unordered.temporal.bindings.reverse();
    assert!(codes(&unordered).contains(&"PLAN_TEMPORAL_ORDER".to_owned()));

    let mut orphan = plan(1);
    orphan
        .temporal
        .programs
        .push(literal_program("tpg_orphan", 0.25, "tpv_main"));
    let mut extra = provenance("extra");
    extra.id = TemporalProvenanceId::new("tpv_extra").unwrap();
    orphan.temporal.provenance.push(extra);
    let actual = codes(&orphan);
    assert!(actual.contains(&"PLAN_TEMPORAL_PROGRAM_ORPHAN".to_owned()));
    assert!(actual.contains(&"PLAN_TEMPORAL_PROVENANCE_ORPHAN".to_owned()));
}

#[test]
fn header_graph_and_cache_identity_are_revalidated_after_decode() {
    let mut value = plan(1);
    value.header.source.executable.core_version = 7;
    value.header.source.semantic_hash = "BAD".to_owned();
    value.header.resolver.resolver_version = "stale".to_owned();
    value.header.cache.temporal_library_sha256 = "0".repeat(64);
    value.output.sequence_id = SequenceId::new("seq_missing").unwrap();
    value.sequences.push(value.sequences[0].clone());
    let actual = codes(&value);
    for expected in [
        "PLAN_EXECUTABLE_MANIFEST",
        "PLAN_SOURCE_DIGEST",
        "PLAN_RESOLVER_IDENTITY",
        "PLAN_CACHE_IDENTITY",
        "PLAN_OUTPUT_SEQUENCE",
        "PLAN_SEQUENCE_DUPLICATE",
        "PLAN_ITEM_DUPLICATE",
    ] {
        assert!(actual.contains(&expected.to_owned()), "missing {expected}");
    }
    assert!(canonical_plan_json(&value).is_err());
}
