#[path = "sinks/content.rs"]
mod content;
#[path = "sinks/fixture.rs"]
mod fixture;
#[path = "sinks/library.rs"]
mod library;

use std::collections::BTreeSet;

use veac_plan::{decode_render_plan_json, resolve, validate_render_plan};

#[test]
fn every_resolved_animation_sink_retains_its_typed_temporal_binding() {
    let (project, expected) = fixture::project();
    let plan = resolve(&project, None).unwrap().remove(0);

    validate_render_plan(&plan).unwrap();
    let actual = plan
        .temporal
        .bindings
        .iter()
        .map(|binding| binding.id.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected.into_iter().collect());
    assert_eq!(plan.temporal.programs.len(), 5);
    assert_eq!(plan.temporal.provenance.len(), 1);

    let encoded = serde_json::to_string(&plan).unwrap();
    assert_eq!(decode_render_plan_json(&encoded).unwrap(), plan);
}
