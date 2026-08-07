use std::fs;

use tempfile::tempdir;
use veac_ir::Animatable;
use veac_lang::program::build_path;

const FACTORY: &str = r#"module {
  export fn card(key: identifier, at: time, color: color) -> Item {
    let value = item(key, item_enabled(), during(at, 1s),
      source_generated(generator_solid(color)), source_timing_native());
    animate visual-opacity on clip(value) { private_curve(progress) }
  }

  fn private_curve(value: scalar) -> scalar {
    clamp(value * 2.0, 0.0, 1.0)
  }
}"#;

const ENTRY: &str = r#"import "./factory.veac" as cards;

fn main(context: Context) -> Project {
  let first = cards.card(identifier("first"), 0s, #c43a69ff);
  let second = cards.card(identifier("second"), 1s, #2d8f85ff);
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(first).with_item(second);
  let timeline = sequence(identifier("main"), "模块组件动画",
    sequence_settings(canvas(320px, 180px), frame_rate(30, 1), 48000))
    .with_layer(visual);
  project(identifier("module-temporal"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}"#;

#[test]
fn exported_factory_retains_private_animation_helper_across_module_boundary() {
    let directory = tempdir().unwrap();
    fs::write(directory.path().join("factory.veac"), FACTORY).unwrap();
    let entry = directory.path().join("main.veac");
    fs::write(&entry, ENTRY).unwrap();
    let built = build_path(&entry).unwrap();
    let envelope = built.envelope();
    assert_eq!(envelope.temporal.programs.len(), 1);
    assert_eq!(envelope.temporal.provenance.len(), 2);
    for clip in &envelope.project.sequences[0].tracks[0].clips {
        assert!(matches!(
            clip.visual.as_ref().unwrap().opacity,
            Animatable::Binding { .. }
        ));
    }
    for provenance in &envelope.temporal.provenance {
        assert_eq!(provenance.definition.source_id.as_str().len(), 68);
        assert!(!provenance.call_stack.is_empty());
        assert!(provenance
            .call_stack
            .iter()
            .any(|site| site.source_id.as_str().starts_with("src_")));
    }
}
