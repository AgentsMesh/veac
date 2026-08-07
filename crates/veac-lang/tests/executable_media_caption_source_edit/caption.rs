use std::fs;

use veac_ir::{ClipSource, FontRef, MaterialSource, TrackKind};
use veac_lang::program::apply_executable_source_edit_path;

use super::support::{batch, Fixture};

#[test]
fn caption_text_and_speaker_edit_rebuilds_atomically_from_source_truth() {
    let fixture = Fixture::new();
    let before = fs::read(&fixture.module).unwrap();
    let body = r#"{ item(identifier("caption"), item_enabled(), during(0s, 2s),
      source_caption_speaker("新字幕", "新旁白", style(primary)),
      source_timing_native()) }"#;
    let preview =
        apply_executable_source_edit_path(&fixture.entry, &batch(&fixture.entry, "caption", body))
            .unwrap();
    let clip = caption_clip(preview.built.envelope());
    let ClipSource::Caption { text, speaker, .. } = &clip.source else {
        panic!("expected caption")
    };
    assert_eq!(text, "新字幕");
    assert_eq!(speaker.as_deref(), Some("新旁白"));
    assert!(preview.source().unwrap().contains("新字幕"));
    assert_eq!(fs::read(&fixture.module).unwrap(), before);
}

#[test]
fn caption_font_reference_edit_selects_the_authored_font_material() {
    let fixture = Fixture::new();
    let body = r#"{ item(identifier("caption"), item_enabled(), during(0s, 2s),
      source_caption_speaker("旧字幕", "旧旁白", style(alternate)),
      source_timing_native()) }"#;
    let preview =
        apply_executable_source_edit_path(&fixture.entry, &batch(&fixture.entry, "caption", body))
            .unwrap();
    let project = &preview.built.envelope().project;
    let expected = project
        .materials
        .iter()
        .find(|material| {
            material.source
                == MaterialSource::File {
                    uri: "assets/alternate.ttf".to_owned(),
                }
        })
        .unwrap();
    let ClipSource::Caption { style, .. } = &caption_clip(preview.built.envelope()).source else {
        panic!("expected caption")
    };
    assert_eq!(
        style.font,
        FontRef::Material {
            material_id: expected.id.clone()
        }
    );
}

fn caption_clip(envelope: &veac_ir::ProjectEnvelope) -> &veac_ir::Clip {
    let track = envelope.project.sequences[0]
        .tracks
        .iter()
        .find(|track| track.kind == TrackKind::Caption)
        .unwrap();
    &track.clips[0]
}
