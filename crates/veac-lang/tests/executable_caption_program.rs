use std::fs;

use tempfile::tempdir;
use veac_ir::{ClipSource, MaterialKind, TrackKind};
use veac_lang::program::{build_path, DomainOperationId as Op};

const ENTRY: &str = r#"import "./library.veac" as library;
fn main(context: Context) -> Project {
  let font = library.font();
  let voice = library.voice();
  let mapped = map(["一", "二"], fn(value: text) -> text effect pure { value });
  let retained_mapped = mapped;
  let first = library.caption(identifier("one"), font);
  let second = library.caption(identifier("two"), font);
  let state = track_state(
    track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked()
  );
  let audio = audio_layer(
    identifier("audio"), 0, placement_free(), state, track_routing_default()
  ).with_item(library.audio(voice));
  let text = caption_layer(
    identifier("captions"), 1, placement_free(), state, track_routing_default()
  ).with_item(first).with_item(second);
  let timeline = sequence(
    identifier("main"), "模块复用字幕",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
  ).with_layer(audio).with_layer(text);
  project(identifier("program"), project_settings(600))
    .with_resource(font).with_resource(voice)
    .with_sequence(timeline).entry(timeline)
}
"#;

const MODULE: &str = r#"module {
  export fn font() -> Resource {
    font_resource(identifier("font"), resource_file("assets/font.ttf"),
      sha256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"))
  }
  export fn voice() -> Resource {
    audio_resource(identifier("voice"), resource_file("assets/voice.wav"),
      sha256("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
      stream_auto())
  }
  export fn audio(source: Resource) -> Item {
    item(identifier("voice"), item_enabled(), during(0s, 2s),
      source_media(source), source_timing_native())
  }
  export fn caption(key: identifier, font: Resource) -> Item {
    let fonts = font_stack(font_resource_ref(font), []);
    let metrics = text_metrics(
      fonts, weight_normal(), font_style_normal(), 30px, 0px, 1.0, #ffffffff
    );
    let layout = text_layout(
      text_box_auto(), text_wrap_none(), text_overflow_visible(),
      text_align_center(), text_align_middle(),
      writing_horizontal_tb(), orientation_mixed()
    );
    let style = text_style(
      metrics, layout, text_path_none(),
      text_decoration(text_background_none(), text_outline_none(), shadow_none()),
      [], text_animation_none()
    );
    item(key, item_enabled(), during(0s, 2s),
      source_caption_speaker("可复用字幕", "旁白", style), source_timing_native())
  }
}
"#;

#[test]
fn modules_functions_closures_and_map_execute_under_v6_topology() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, ENTRY).unwrap();
    fs::write(temp.path().join("library.veac"), MODULE).unwrap();
    let built = build_path(&entry).unwrap();
    let project = &built.envelope().project;
    assert_eq!(project.materials.len(), 2);
    assert!(project
        .materials
        .iter()
        .any(|material| material.kind == MaterialKind::Audio));
    assert!(project
        .materials
        .iter()
        .any(|material| material.kind == MaterialKind::Font));
    for material in &project.materials {
        let event = &material.authorship.as_ref().unwrap().events[0];
        assert_eq!(event.origin.source.as_str(), "library.veac");
        assert!(
            [Op::AudioResource.opcode(), Op::FontResource.opcode()].contains(&event.operation.0)
        );
    }

    let sequence = &project.sequences[0];
    assert_eq!(sequence.tracks[0].kind, TrackKind::Audio);
    assert_eq!(sequence.tracks[1].kind, TrackKind::Caption);
    assert_constructor(&sequence.tracks[0].clips[0], Op::Item.opcode(), false);
    assert!(matches!(
        sequence.tracks[0].clips[0].source,
        ClipSource::Media { .. }
    ));
    assert_eq!(sequence.tracks[1].clips.len(), 2);
    for clip in &sequence.tracks[1].clips {
        assert_constructor(clip, Op::Item.opcode(), false);
        let ClipSource::Caption { speaker, style, .. } = &clip.source else {
            panic!("expected caption source");
        };
        assert_eq!(speaker.as_deref(), Some("旁白"));
        assert_eq!(style.size_pixels, 30.0);
    }
    veac_ir::validate(built.envelope()).unwrap();
}

fn assert_constructor(clip: &veac_ir::Clip, operation: u16, closure: bool) {
    let event = &clip.authorship.as_ref().unwrap().events[0];
    assert_eq!(event.operation.0, operation);
    assert_eq!(event.origin.source.as_str(), "library.veac");
    let stack = &event.call_stack;
    assert_eq!(stack[0].function.as_str(), "main");
    assert_eq!(
        stack
            .iter()
            .any(|frame| frame.function.as_str() == "<closure>"),
        closure
    );
}
