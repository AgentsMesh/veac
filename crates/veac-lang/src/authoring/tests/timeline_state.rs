use crate::authoring::{format_document, lower_document, parse};

use super::project;

#[test]
fn state_round_trips_and_lowers_independent_axes() {
    let source = timeline(
        "state { audio muted; isolation solo; }",
        "state { playback disabled; }",
        "",
    );
    let parsed = parse(&source).unwrap();
    let formatted = format_document(&parsed);
    let reparsed = parse(&formatted).unwrap();
    assert_eq!(format_document(&reparsed), formatted);

    let envelope = lower_document(&reparsed).unwrap();
    let track = &envelope.project.sequences[0].tracks[0];
    assert!(track.state.enabled);
    assert!(track.state.muted);
    assert!(track.state.solo);
    assert!(!track.state.locked);
    assert!(!track.clips[0].enabled);
}

#[test]
fn omitted_state_axes_equal_explicit_defaults() {
    let omitted = lower_track(&timeline("", "", ""));
    let explicit = lower_track(&timeline(
        "state { playback enabled; audio audible; isolation normal; editing editable; }",
        "",
        "",
    ));
    assert_eq!(omitted.state, explicit.state);
}

#[test]
fn empty_track_state_is_rejected() {
    let diagnostics = parse(&timeline("state {}", "", "")).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_TRACK_STATE_EMPTY"));
}

#[test]
fn audio_layers_reject_each_visual_modifier_domain() {
    for modifier in [MASK, COLOR, SURFACE, VIDEO_EFFECT] {
        let parsed = parse(&timeline("", "", modifier)).unwrap();
        let diagnostics = lower_document(&parsed).unwrap_err();
        assert!(
            diagnostics
                .as_slice()
                .iter()
                .any(|value| value.code == "AUTHORING_LOWER_TRACK_COMPONENT"),
            "visual modifier was accepted: {modifier}"
        );
    }
}

#[test]
fn audio_layers_accept_audio_modifier_and_audio_effect() {
    let modifiers = r#"
      audio mix { gain 0db; }
      effect normalize { type audio.normalize; parameter target_lufs -18; }
    "#;
    let envelope = lower_document(&parse(&timeline("", "", modifiers)).unwrap()).unwrap();
    let clip = &envelope.project.sequences[0].tracks[0].clips[0];
    assert!(clip.audio.is_some());
    assert_eq!(clip.effects[0].effect_type, "audio.normalize");
}

fn lower_track(source: &str) -> veac_ir::Track {
    lower_document(&parse(source).unwrap())
        .unwrap()
        .project
        .sequences[0]
        .tracks[0]
        .clone()
}

fn timeline(track_state: &str, item_state: &str, modifiers: &str) -> String {
    project(&format!(
        r#"sequence main {{
  layer audio sound {{
    {track_state}
    item tone {{
      source generated silence;
      record {{ at 0s; duration 1s; }}
      {item_state}
      modifiers {{ {modifiers} }}
    }}
  }}
}}"#
    ))
}

const MASK: &str = "mask visual { shape circle; }";
const SURFACE: &str = "surface visual { corner-radius 1px; }";
const VIDEO_EFFECT: &str = "effect visual { type video.blur; parameter radius 1; }";
const COLOR: &str = r#"color visual {
  input-space { primaries bt709; transfer bt709; matrix bt709; range limited; }
  working-space { primaries bt709; transfer linear; matrix rgb; range full; }
  output-space { primaries bt709; transfer bt709; matrix bt709; range limited; }
}"#;
