use std::collections::BTreeSet;

use veac_lang::program::build_source;

const SOURCE: &str = r#"
fn card(key: identifier) -> Item {
  item(key, item_enabled(), during(0s, 1s),
    source_generated(generator_solid(#c43a69ff)), source_timing_native())
}

fn animate_card(value: Item) -> Item {
  animate visual-opacity on clip(value) { progress }
}

fn main(context: Context) -> Project {
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let first = card(identifier("first"));
  let second = card(identifier("second"));
  let animated = for value in [first, second] {
    animate_card(value)
  };
  let retained = animated;
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(first).with_item(second);
  let timeline = sequence(identifier("main"), "迭代组件动画",
    sequence_settings(canvas(320px, 180px), frame_rate(30, 1), 48000))
    .with_layer(visual);
  project(identifier("iteration-temporal"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#;

#[test]
fn iteration_instances_have_stable_distinct_logical_and_provenance_identities() {
    let first = build_source(SOURCE).unwrap();
    let second = build_source(SOURCE).unwrap();
    let first = &first.envelope().temporal;
    let second = &second.envelope().temporal;
    assert_eq!(first, second);
    assert_eq!(first.bindings.len(), 2);
    assert_eq!(
        first
            .provenance
            .iter()
            .flat_map(|value| &value.logical_keys)
            .collect::<BTreeSet<_>>()
            .len(),
        2
    );
    assert!(first
        .provenance
        .iter()
        .all(|value| !value.call_stack.is_empty()));
}
