use std::fmt::Write;
use veac_ir::{Clip, ProjectEnvelope};
use veac_lang::program::{build_source, BuiltProgram, Diagnostic};

pub(super) fn build(source: &str) -> BuiltProgram {
    build_source(source).unwrap_or_else(|errors| panic!("unexpected diagnostics: {errors}"))
}

pub(super) fn envelope(source: &str) -> ProjectEnvelope {
    build(source).envelope().clone()
}

pub(super) fn error(source: &str) -> Diagnostic {
    build_source(source)
        .expect_err("source should fail executable lowering")
        .as_slice()
        .first()
        .expect("one bounded diagnostic")
        .clone()
}

pub(super) fn clip(envelope: &ProjectEnvelope, index: usize) -> &Clip {
    &envelope.project.sequences[0].tracks[0].clips[index]
}

pub(super) fn visual_project(items: &[&str]) -> String {
    visual_project_with_timebase(items, 600)
}

pub(super) fn visual_project_with_timebase(items: &[&str], timebase: u32) -> String {
    let mut attachments = String::new();
    for item in items {
        write!(attachments, ".with_item({item})").unwrap();
    }
    format!(
        r#"
fn main(context: Context) -> Project {{
    let state = track_state(track_playback_enabled(), track_audio_audible(),
        track_isolation_normal(), track_editing_unlocked());
    let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
        track_routing_default()){attachments};
    let timeline = sequence(identifier("main"), "视觉测试",
        sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
        .with_layer(visual);
    project(identifier("demo"), project_settings({timebase}))
        .with_sequence(timeline).entry(timeline)
}}
"#
    )
}

pub(super) fn solid(key: &str, color: &str, start: &str, duration: &str) -> String {
    format!(
        "item(identifier(\"{key}\"), item_enabled(), during({start}, {duration}), \
        source_generated(generator_solid({color})), source_timing_native())"
    )
}
