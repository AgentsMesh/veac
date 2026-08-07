use std::fs;

use tempfile::tempdir;
use veac_lang::program::build_path;

const NESTED: &str = r#"fn main(context: Context) -> Project {
  let seed = item(identifier("seed"), item_enabled(), during(0s, 1s),
    source_generated(generator_solid(#112233ff)), source_timing_native());
  let nested = for outer_key in [identifier("outer")] {
    for inner_key in [identifier("generated")] {
      item(inner_key, item_enabled(), during(1s, 1s),
        source_generated(generator_solid(#445566ff)), source_timing_native())
    }
  };
  let generated_list = fold(nested, [seed],
    fn(current: list<Item>, next: list<Item>) -> list<Item> effect pure { next });
  let generated = fold(generated_list, seed,
    fn(current: Item, next: Item) -> Item effect pure { next });
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let layer = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(seed).with_item(generated);
  let timeline = sequence(identifier("main"), "嵌套迭代来源",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
    .with_layer(layer);
  project(identifier("project"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#;

const FAILURE: &str = r#"fn main(context: Context) -> Project {
  let first = item(identifier("first"), item_enabled(), during(0s, 1s),
    source_generated(generator_solid(#112233ff)), source_timing_native());
  let second = item(identifier("second"), item_enabled(), during(1s, 1s),
    source_generated(generator_solid(#445566ff)), source_timing_native());
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let base = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default());
  let attempts = for group_key in [identifier("group")] {
    for candidate in [first, second] { base.with_item(candidate) }
  };
  let retained = attempts;
  let timeline = sequence(identifier("main"), "失败迭代",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000));
  project(identifier("project"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#;

#[test]
fn nested_effectful_for_reaches_graph_entity_provenance() {
    let first = build(NESTED);
    let second = build(NESTED);
    let event = generated_event(&first);
    let repeated = generated_event(&second);
    assert_eq!(event.iterations, repeated.iterations);
    assert_eq!(
        generated_provenance(&first)
            .logical_path
            .last()
            .unwrap()
            .as_str(),
        "generated"
    );

    let loops = &event.iterations;
    assert_eq!(loops.len(), 2);
    assert_loop(&loops[0], NESTED, "outer_key", 0);
    assert_loop(&loops[1], NESTED, "inner_key", 0);
    assert_ne!(loops[0].logical_key, loops[1].logical_key);
}

#[test]
fn failed_effectful_for_reports_index_key_and_binding_span() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, FAILURE).unwrap();
    let diagnostics = build_path(&entry).unwrap_err();
    let error = &diagnostics.as_slice()[0];
    assert_eq!(error.code, "PROGRAM_EXECUTABLE_RUNTIME");
    assert!(error.message.contains("DOMAIN_HANDLE_STALE"));
    assert!(error.message.contains("index = 0"));
    assert!(error.message.contains("index = 1"));
    assert_eq!(error.message.matches("; loop `loop_").count(), 2);
    let mut binding_offsets = Vec::new();
    for binding in ["group_key", "candidate"] {
        let start = FAILURE.find(&format!("{binding} in")).unwrap();
        let end = start + binding.len();
        let site = format!("main.veac:{start}..{end}");
        binding_offsets.push(error.message.find(&site).unwrap());
    }
    assert!(binding_offsets[0] < binding_offsets[1]);
}

fn build(source: &str) -> veac_ir::ProjectEnvelope {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, source).unwrap();
    build_path(&entry).unwrap().envelope().clone()
}

fn generated_event(envelope: &veac_ir::ProjectEnvelope) -> &veac_ir::AuthorshipEvent {
    &generated_provenance(envelope).events[0]
}

fn generated_provenance(envelope: &veac_ir::ProjectEnvelope) -> &veac_ir::EntityAuthorship {
    envelope.project.sequences[0].tracks[0].clips[1]
        .authorship
        .as_ref()
        .unwrap()
}

fn assert_loop(value: &veac_ir::AuthoredIteration, source: &str, binding: &str, index: u64) {
    let start = source.find(&format!("{binding} in")).unwrap();
    let end = start + binding.len();
    assert_eq!(value.index, index);
    assert_eq!(value.binding_span.start, start as u64);
    assert_eq!(value.binding_span.end, end as u64);
    assert_eq!(&source[start..end], binding);
    let key = value.logical_key.as_str();
    assert!(key.starts_with("loop_"));
    assert_eq!(key.len(), 69);
}
