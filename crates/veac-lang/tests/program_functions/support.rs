#![allow(dead_code)]

use std::fmt::Write;
use veac_lang::program::BuiltProgram;

const PROJECT_BODY: &str = r#"
fn main(context: Context) -> Project {
  let state = track_state(
    track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked()
  );
  let layer = visual_layer(
    identifier("content"), 0, placement_free(), state, track_routing_default()
  )$ITEMS;
  let timeline = sequence(
    identifier("main"), "函数测试",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
  ).with_layer(layer);
  project(identifier("functions"), project_settings(1000))
    .with_sequence(timeline).entry(timeline)
}"#;

pub fn project_with(declarations: &str, duration: &str) -> String {
    project_with_durations(declarations, &[duration])
}

pub fn project_with_durations(declarations: &str, durations: &[&str]) -> String {
    let mut items = String::new();
    for (index, duration) in durations.iter().enumerate() {
        write!(
            items,
            ".with_item(item(identifier(\"result-{index}\"), item_enabled(), \
             during(0s, {duration}), source_generated(generator_transparent()), \
             source_timing_native()))"
        )
        .unwrap();
    }
    format!("{declarations}\n{}", PROJECT_BODY.replace("$ITEMS", &items))
}

pub fn item_duration(program: &BuiltProgram, sequence: usize, track: usize, clip: usize) -> String {
    let duration = program.envelope().project.sequences[sequence].tracks[track].clips[clip]
        .record_range
        .duration;
    assert_eq!(duration.timescale, 1000);
    if duration.value % 1000 == 0 {
        format!("{}s", duration.value / 1000)
    } else {
        format!("{}ms", duration.value)
    }
}

pub fn result_duration(program: &BuiltProgram) -> String {
    item_duration(program, 0, 0, 0)
}

pub fn validate_canonical(program: &BuiltProgram) {
    veac_ir::validate(program.envelope()).unwrap();
}
