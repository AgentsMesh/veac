use std::collections::BTreeMap;

use veac_ir::{ClipSource, Generator};

use super::support;

#[test]
fn reordering_attachment_lists_does_not_rename_items() {
    let first = support::solid("first", "#ff0000ff", "0s", "1s");
    let second = support::solid("second", "#00ff00ff", "1s", "1s");
    let left = support::envelope(&support::visual_project(&[&first, &second]));
    let right = support::envelope(&support::visual_project(&[&second, &first]));
    assert_eq!(colors_to_ids(&left), colors_to_ids(&right));
}

#[test]
fn equal_leaf_keys_under_different_parents_have_distinct_ids() {
    let source = r#"
fn main(context: Context) -> Project {
    let state = track_state(track_playback_enabled(), track_audio_audible(),
        track_isolation_normal(), track_editing_unlocked());
    let first = item(identifier("same"), item_enabled(), during(0s, 1s),
        source_generated(generator_transparent()), source_timing_native());
    let second = item(identifier("same"), item_enabled(), during(0s, 1s),
        source_generated(generator_transparent()), source_timing_native());
    let main_layer = visual_layer(identifier("content"), 0, placement_free(), state,
        track_routing_default()).with_item(first);
    let other_layer = visual_layer(identifier("content"), 0, placement_free(), state,
        track_routing_default()).with_item(second);
    let settings = sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000);
    let main = sequence(identifier("main"), "主时间线", settings).with_layer(main_layer);
    let other = sequence(identifier("other"), "其他时间线", settings).with_layer(other_layer);
    project(identifier("demo"), project_settings(600))
        .with_sequence(main).with_sequence(other).entry(main)
}
"#;
    let envelope = support::envelope(source);
    let tracks = envelope
        .project
        .sequences
        .iter()
        .map(|sequence| &sequence.tracks[0])
        .collect::<Vec<_>>();
    assert_ne!(tracks[0].id, tracks[1].id);
    assert_ne!(tracks[0].clips[0].id, tracks[1].clips[0].id);
}

fn colors_to_ids(envelope: &veac_ir::ProjectEnvelope) -> BTreeMap<(u8, u8, u8), String> {
    envelope.project.sequences[0].tracks[0]
        .clips
        .iter()
        .map(|clip| {
            let ClipSource::Generated {
                generator: Generator::Solid { color },
            } = &clip.source
            else {
                panic!("expected solid generator")
            };
            ((color.red, color.green, color.blue), clip.id.to_string())
        })
        .collect()
}
