use veac_ir::{ClipSource, Generator, PlacementMode, TrackKind, CURRENT_SCHEMA_VERSION};

use super::support;

const SOURCE: &str = r#"
fn title_style(font: Resource) -> TextStyle {
    let fonts = font_stack(font_resource_ref(font), []);
    let metrics = text_metrics(
        fonts, weight_normal(), font_style_normal(), 36px, 0px, 1.0, #f0e0d0cc
    );
    let layout = text_layout(
        text_box_auto(), text_wrap_none(), text_overflow_visible(),
        text_align_center(), text_align_middle(),
        writing_horizontal_tb(), orientation_mixed()
    );
    text_style(metrics, layout, text_path_none(),
        text_decoration(text_background_none(), text_outline_none(), shadow_none()),
        [], text_animation_none())
}
fn main(context: Context) -> Project {
    let font = font_resource(identifier("font"), resource_file("assets/font.ttf"),
        sha256("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"));
    let background = item(identifier("background"), item_enabled(), during(0s, 3s),
        source_generated(generator_solid(#112233ff)), source_timing_native());
    let title = item(identifier("title"), item_enabled(), during(1s, 2s),
        source_text("Direct IR", title_style(font)), source_timing_native());
    let state = track_state(track_playback_enabled(), track_audio_audible(),
        track_isolation_normal(), track_editing_unlocked());
    let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
        track_routing_default()).with_item(background).with_item(title);
    let audio = audio_layer(identifier("audio"), 1, placement_free(), state,
        track_routing_default());
    let timeline = sequence(identifier("main"), "规范 IR",
        sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
        .with_layer(visual).with_layer(audio);
    project(identifier("demo"), project_settings(600)).with_resource(font)
        .with_sequence(timeline).entry(timeline)
}
"#;

#[test]
fn executable_graph_lowers_directly_to_current_canonical_ir() {
    let built = support::build(SOURCE);
    let envelope = built.envelope();
    assert_eq!(envelope.schema_version, CURRENT_SCHEMA_VERSION);
    assert_eq!(envelope.project.timebase, 600);
    assert_eq!(envelope.project.sequences.len(), 1);
    assert_eq!(envelope.project.sequences[0].settings.width, 640);
    assert_eq!(envelope.project.sequences[0].settings.height, 360);
    assert_eq!(envelope.project.sequences[0].settings.sample_rate, 48_000);
    assert_eq!(envelope.project.sequences[0].tracks.len(), 2);
    assert_eq!(
        envelope.project.sequences[0].tracks[0].kind,
        TrackKind::Visual
    );
    assert_eq!(
        envelope.project.sequences[0].tracks[1].kind,
        TrackKind::Audio
    );
    assert_eq!(
        envelope.project.sequences[0].tracks[0].placement_mode,
        PlacementMode::Free
    );
    assert!(envelope.project.sequences[0].tracks[1].clips.is_empty());
    assert!(veac_ir::validate(envelope).is_ok());
}

#[test]
fn generated_and_text_clips_preserve_authored_mechanisms() {
    let envelope = support::envelope(SOURCE);
    let background = support::clip(&envelope, 0);
    assert!(matches!(
        background.source,
        ClipSource::Generated {
            generator: Generator::Solid { .. }
        }
    ));
    assert!(background.visual.is_some());
    let title = support::clip(&envelope, 1);
    let ClipSource::Text { text, style } = &title.source else {
        panic!("expected text source")
    };
    assert_eq!(text, "Direct IR");
    assert_eq!(style.size_pixels, 36.0);
    assert_eq!((style.color.red, style.color.green), (240, 224));
    assert_eq!((style.color.blue, style.color.alpha), (208, 204));
    assert!(title.visual.is_some());
}

#[test]
fn v6_does_not_invent_an_implicit_preview_delivery() {
    let envelope = support::envelope(SOURCE);
    assert!(envelope.project.render_configs.is_empty());
}

#[test]
fn canonical_json_roundtrips_without_semantic_loss() {
    let envelope = support::envelope(SOURCE);
    let json = veac_ir::canonical_json(&envelope).expect("valid canonical JSON");
    let decoded = veac_ir::decode_canonical_json(&json).expect("canonical JSON decodes");
    assert_eq!(decoded, envelope);
}

#[test]
fn repeated_execution_publishes_identical_canonical_envelopes() {
    let first = support::envelope(SOURCE);
    let second = support::envelope(SOURCE);
    assert_eq!(first, second);
}
