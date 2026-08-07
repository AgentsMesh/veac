use std::fs;

use tempfile::tempdir;
use veac_lang::program::{build_path, build_source, BuiltProgram};

fn project(declarations: &str, duration: &str) -> String {
    format!(
        r#"{declarations}
fn main(context: Context) -> Project {{
  let result = item(identifier("result"), item_enabled(), during(0s, {duration}),
    source_generated(generator_solid(#00000000)), source_timing_native());
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(result);
  let timeline = sequence(identifier("main"), "名义类型执行",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
    .with_layer(visual);
  project(identifier("nominal-execution"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}}"#
    )
}

fn duration(program: &BuiltProgram) -> veac_ir::RationalTime {
    program.envelope().project.sequences[0].tracks[0].clips[0]
        .record_range
        .duration
}

fn validate(program: &BuiltProgram) {
    veac_ir::validate(program.envelope()).unwrap();
}

#[test]
fn constructors_projections_and_exhaustive_match_execute() {
    let declarations = r#"struct Timing { start: time, duration: time, }
enum CardPlacement { Center, Corner { x: length, y: length, }, }
fn finish(timing: Timing, placement: CardPlacement) -> time {
  let inset = match placement {
    CardPlacement.Center => 0px,
    CardPlacement.Corner { x, y: vertical } => x + vertical,
  };
  if inset == 20px { timing.start + timing.duration } else { 1s }
}"#;
    let compiled = build_source(&project(
        declarations,
        "finish(Timing { duration: 2s, start: 1s, }, CardPlacement.Corner { y: 12px, x: 8px, })",
    ))
    .unwrap();
    assert_eq!(duration(&compiled).value, 1_800);
    validate(&compiled);
}

#[test]
fn unselected_wildcard_arm_does_not_execute() {
    let declarations = r#"enum Choice { Exact { duration: time, }, Fallback, }
fn choose(value: Choice) -> time {
  match value {
    Choice.Exact { duration } => duration,
    _ => 1s / 0.0,
  }
}"#;
    let compiled = build_source(&project(
        declarations,
        "choose(Choice.Exact { duration: 750ms, })",
    ))
    .unwrap();
    assert_eq!(duration(&compiled).value, 450);
    validate(&compiled);
}

#[test]
fn authored_match_order_and_wildcard_lower_to_canonical_variant_order() {
    let declarations = r#"enum Choice { First, Second { duration: time, }, Third, }
fn explicit(value: Choice) -> time {
  match value {
    Choice.Third => 3s,
    Choice.Second { duration } => duration,
    Choice.First => 1s,
  }
}
fn fallback(value: Choice) -> time {
  match value {
    Choice.Second { duration } => duration,
    _ => 1s,
  }
}"#;
    let compiled = build_source(&project(
        declarations,
        "explicit(Choice.Second { duration: 2s, }) + fallback(Choice.Second { duration: 2s, })",
    ))
    .unwrap();
    assert_eq!(duration(&compiled).value, 2_400);
    validate(&compiled);
}

#[test]
fn enum_payload_patterns_must_bind_every_field() {
    let declarations = r#"enum Choice { Pair { first: time, second: time, }, }
fn choose(value: Choice) -> time {
  match value { Choice.Pair { first } => first, }
}"#;
    let error = build_source(&project(
        declarations,
        "choose(Choice.Pair { first: 1s, second: 2s, })",
    ))
    .unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_FUNCTION_EXPRESSION");
    assert!(error.as_slice()[0]
        .message
        .contains("EXPRESSION_INCOMPLETE_PATTERN"));
}

#[test]
fn imported_nominal_function_reaches_private_helper() {
    let module = r#"module {
  export struct Timing { start: time, duration: time, }
  export fn finish(value: Timing) -> time { private(value) }
  fn private(value: Timing) -> time { value.start + value.duration }
}"#;
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("timing.veac"), module).unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(
        &entry,
        project(
            r#"import "./timing.veac" as timing;"#,
            "timing.finish(timing.Timing { duration: 2s, start: 1s, })",
        ),
    )
    .unwrap();

    let compiled = build_path(&entry).unwrap();
    assert_eq!(duration(&compiled).value, 1_800);
    validate(&compiled);
}
