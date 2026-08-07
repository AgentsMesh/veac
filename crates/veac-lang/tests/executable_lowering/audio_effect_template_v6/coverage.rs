use veac_ir::{PlacementMode, TrackRouting};

use super::super::support;
use super::SOURCE;

#[test]
fn magnetic_tracks_all_state_flags_and_bus_routing_lower() {
    let source = SOURCE
        .replace("placement_free()", "placement_magnetic()")
        .replace("track_playback_enabled()", "track_playback_disabled()")
        .replace("track_audio_audible()", "track_audio_muted()")
        .replace("track_isolation_normal()", "track_isolation_solo()")
        .replace("track_editing_unlocked()", "track_editing_locked()")
        .replace(
            "let visuals = visual_layer(",
            "let bus = audio_bus(identifier(\"mix\"));\n    let visuals = visual_layer(",
        );
    let marker = "let audio_track = audio_layer(\n        identifier(\"audio\"), 1, placement_magnetic(), state, track_routing_default()";
    let source = source.replace(
        marker,
        "let audio_track = audio_layer(\n        identifier(\"audio\"), 1, placement_magnetic(), state, track_routing_bus(bus)",
    );
    let envelope = support::envelope(&source);
    let tracks = &envelope.project.sequences[0].tracks;
    assert!(tracks
        .iter()
        .all(|track| track.placement_mode == PlacementMode::Magnetic));
    assert!(tracks.iter().all(|track| {
        !track.state.enabled && track.state.muted && track.state.solo && track.state.locked
    }));
    assert!(matches!(tracks[1].routing, TrackRouting::AudioBus { .. }));
}

#[test]
fn processor_integer_and_percent_boundaries_fail_in_lowering() {
    for (from, to) in [
        ("80.0, 0.7, 2", "80.0, 0.7, -1"),
        ("80.0, 0.7, 2", "80.0, 0.7, 256"),
        ("18000.0, 0.7, 2", "18000.0, 0.7, -1"),
        ("18000.0, 0.7, 2", "18000.0, 0.7, 256"),
        ("4.0, 2.0, 75%", "4.0, 2.0, 101%"),
    ] {
        let error = support::error(&SOURCE.replacen(from, to, 1));
        assert_eq!(error.code, "PROGRAM_EXECUTABLE_LOWER", "{to}");
    }
}
