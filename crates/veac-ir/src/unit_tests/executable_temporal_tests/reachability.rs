use crate::{test_support::sample_project, *};

use super::support::{add_progress_clock, bind, binding, codes, provenance};

fn bind_video_opacity(project: &mut ProjectEnvelope, value_type: TemporalType) {
    let id = bind(project, value_type, "opacity");
    project.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .opacity = binding(&id);
}

#[test]
fn reachable_typed_binding_program_and_provenance_validate() {
    let mut project = sample_project();
    bind_video_opacity(&mut project, TemporalType::Scalar);
    add_progress_clock(&mut project, "itm_video");
    project.temporal.programs[0].nodes[0].provenance_id =
        Some(TemporalProvenanceId::new("tpv_main").unwrap());
    validate(&project).unwrap();
    assert!(canonical_json(&project).unwrap().contains("tbd_opacity"));
}

#[test]
fn sinks_reject_missing_binding_and_both_type_mismatches() {
    let mut missing = sample_project();
    missing.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .opacity = binding(&TemporalBindingId::new("tbd_missing").unwrap());
    assert!(codes(&missing).contains(&"TEMPORAL_SINK_BINDING_MISSING".to_owned()));

    let mut mismatch = sample_project();
    bind_video_opacity(&mut mismatch, TemporalType::Vec2);
    let actual = codes(&mismatch);
    assert!(actual.contains(&"TEMPORAL_SINK_TYPE".to_owned()));
    assert!(actual.contains(&"TEMPORAL_SINK_PROGRAM_TYPE".to_owned()));
}

#[test]
fn clock_owners_must_exist_and_match_the_sink() {
    let mut mismatch = sample_project();
    bind_video_opacity(&mut mismatch, TemporalType::Scalar);
    add_progress_clock(&mut mismatch, "itm_caption");
    let actual = codes(&mismatch);
    assert!(actual.contains(&"TEMPORAL_SINK_CLOCK_OWNER".to_owned()));
    assert!(!actual.contains(&"TEMPORAL_CLOCK_OWNER_MISSING".to_owned()));

    let mut missing = sample_project();
    bind_video_opacity(&mut missing, TemporalType::Scalar);
    add_progress_clock(&mut missing, "itm_missing");
    let actual = codes(&missing);
    assert!(actual.contains(&"TEMPORAL_CLOCK_OWNER_MISSING".to_owned()));
    assert!(actual.contains(&"TEMPORAL_SINK_CLOCK_OWNER".to_owned()));
}

#[test]
fn every_unreachable_library_record_is_rejected() {
    let mut project = sample_project();
    bind(&mut project, TemporalType::Scalar, "orphan");
    let mut extra = provenance();
    extra.id = TemporalProvenanceId::new("tpv_orphan").unwrap();
    project.temporal.provenance.push(extra);
    let actual = codes(&project);
    for expected in [
        "TEMPORAL_BINDING_ORPHAN",
        "TEMPORAL_PROGRAM_ORPHAN",
        "TEMPORAL_PROVENANCE_ORPHAN",
    ] {
        assert!(actual.contains(&expected.to_owned()));
    }
}

#[test]
fn duplicate_and_broken_library_records_surface_in_canonical_validation() {
    let mut project = sample_project();
    bind_video_opacity(&mut project, TemporalType::Scalar);
    project
        .temporal
        .programs
        .push(project.temporal.programs[0].clone());
    project
        .temporal
        .bindings
        .push(project.temporal.bindings[0].clone());
    project
        .temporal
        .provenance
        .push(project.temporal.provenance[0].clone());
    let actual = codes(&project);
    for expected in [
        "TEMPORAL_PROGRAM_DUPLICATE",
        "TEMPORAL_PROGRAM_CONTENT_DUPLICATE",
        "TEMPORAL_BINDING_DUPLICATE",
        "TEMPORAL_PROVENANCE_DUPLICATE",
    ] {
        assert!(actual.contains(&expected.to_owned()));
    }
}
