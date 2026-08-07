use super::support::*;

const SOURCE: &str = r#"
fn pulse(value: scalar) -> scalar { clamp(value * 2.0, 0.0, 1.0) }

animate visual-opacity on clip(@cli-temporal, @main, @visual, @first) {
  pulse(progress)
}

fn main(context: Context) -> Project {
  let first = item(identifier("first"), item_enabled(), during(0s, 2s),
    source_generated(generator_solid(#234567ff)), source_timing_native());
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let layer = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(first);
  let timeline = sequence(identifier("main"), "命令行动画",
    sequence_settings(canvas(64px, 36px), frame_rate(30, 1), 48000))
    .with_layer(layer);
  project(identifier("cli-temporal"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#;

#[test]
fn check_and_build_publish_authored_temporal_programs() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, SOURCE);
    let ir = temp.path().join("project.json");
    veac()
        .args(["check", source.to_str().unwrap()])
        .assert()
        .success();
    veac()
        .args(["build", source.to_str().unwrap(), "--emit-ir"])
        .arg(&ir)
        .assert()
        .success();
    let envelope: veac_ir::ProjectEnvelope =
        serde_json::from_str(&std::fs::read_to_string(ir).unwrap()).unwrap();
    assert_eq!(envelope.temporal.programs.len(), 1);
    assert_eq!(envelope.temporal.bindings.len(), 1);
    let clip = &envelope.project.sequences[0].tracks[0].clips[0];
    assert!(matches!(
        clip.visual.as_ref().unwrap().opacity,
        veac_ir::Animatable::Binding { .. }
    ));
}

#[test]
fn source_index_exposes_the_typed_temporal_body_site() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, SOURCE);
    let output = veac()
        .args(["source-index", source.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json = String::from_utf8(output.stdout).unwrap();
    assert!(json.contains("temporal_animation"));
    assert!(json.contains("visual_opacity"));
    assert!(json.contains("first"));
}
