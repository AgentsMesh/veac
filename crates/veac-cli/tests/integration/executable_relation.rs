use super::support::*;

const SOURCE: &str = r#"
fn main(context: Context) -> Project {
  let outgoing = item(identifier("out"), item_enabled(), during(0s, 1500ms),
    source_generated(generator_transparent()), source_timing_native());
  let incoming = item(identifier("in"), item_enabled(), during(1s, 1s),
    source_generated(generator_transparent()), source_timing_native());
  let cross = relation_transition(identifier("cross"), outgoing, incoming,
    transition_dissolve(500ms));
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(outgoing).with_item(incoming);
  let timeline = sequence(identifier("main"), "CLI relation",
    sequence_settings(canvas(64px, 36px), frame_rate(10, 1), 48000))
    .with_layer(visual).with_relation(cross);
  project(identifier("cli-relation"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#;

#[test]
fn cli_check_and_build_publish_canonical_transition_relation() {
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
    let bytes = std::fs::read(ir).unwrap();
    let project: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let relation = &project["project"]["relations"][0];
    assert_eq!(relation["kind"]["type"], "transition");
    assert_eq!(relation["kind"]["transition"]["kind"]["type"], "dissolve");
    assert_eq!(relation["kind"]["transition"]["alignment"], "centered");
    assert_eq!(relation["kind"]["transition"]["duration"]["value"], 300);
    assert_eq!(relation["kind"]["transition"]["duration"]["timescale"], 600);
}
