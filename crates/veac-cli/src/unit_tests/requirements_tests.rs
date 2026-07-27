use std::collections::BTreeSet;

use tempfile::tempdir;
use veac_ir::{
    AudioCodec, AudioOutput, ClipSource, Color, FontRef, Generator, ItemId, MaterialId,
    RationalTime, RenderConfigId, SequenceId, TextSpan, TextStyle, TrackId, TrackKind,
};

use super::support::{canonical_project, MEDIA_SOURCE};

#[path = "requirements_tests/graph_tests.rs"]
mod graph_tests;

#[test]
fn collector_folds_solo_disabled_and_audio_output_state() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    let footage = envelope.project.materials[0].id.clone();
    add_material(&mut envelope, "med_disabled");
    add_material(&mut envelope, "med_skipped");
    add_material(&mut envelope, "med_voice");

    let base = envelope.project.sequences[0].tracks[0].clone();
    let mut chosen = base.clone();
    chosen.state.solo = true;
    let mut disabled = chosen.clips[0].clone();
    disabled.id = ItemId::new("itm_disabled").unwrap();
    disabled.enabled = false;
    disabled.source = media("med_disabled");
    chosen.clips.push(disabled);
    let mut skipped = base.clone();
    skipped.id = TrackId::new("trk_skipped").unwrap();
    skipped.order = 1;
    skipped.clips[0].source = media("med_skipped");
    let mut audio = base;
    audio.id = TrackId::new("trk_audio").unwrap();
    audio.order = 2;
    audio.kind = TrackKind::Audio;
    audio.clips[0].source = media("med_voice");
    envelope.project.sequences[0].tracks = vec![chosen, skipped, audio];

    assert_eq!(required(&envelope), BTreeSet::from([footage.clone()]));
    envelope.project.sequences[0].tracks[0].state.solo = false;
    envelope.project.sequences[0].tracks[1].state.enabled = false;
    assert_eq!(required(&envelope), BTreeSet::from([footage.clone()]));
    envelope.project.render_configs[0]
        .video_deliverable_mut()
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
    assert_eq!(
        required(&envelope),
        BTreeSet::from([footage, MaterialId::new("med_voice").unwrap()])
    );
}

#[test]
fn collector_traverses_nested_freeze_font_and_cycle_sources() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    let material = envelope.project.materials[0].id.clone();
    let mut nested = envelope.project.sequences[0].clone();
    nested.id = SequenceId::new("seq_nested").unwrap();
    nested.name = "nested".into();
    envelope.project.sequences[0].tracks[0].clips[0].source = ClipSource::Sequence {
        sequence_id: nested.id.clone(),
    };
    envelope.project.sequences[0].tracks[0].clips[0].source_mapping = None;
    envelope.project.sequences.push(nested);
    assert_eq!(required(&envelope), BTreeSet::from([material.clone()]));

    envelope.project.sequences[1].tracks[0].clips[0].source = ClipSource::FreezeFrame {
        material_id: material.clone(),
        source_time: RationalTime::zero(600).unwrap(),
    };
    assert_eq!(required(&envelope), BTreeSet::from([material.clone()]));
    envelope.project.sequences[1].tracks[0].clips[0].source = ClipSource::Text {
        text: "title".into(),
        style: TextStyle {
            font: FontRef::Material {
                material_id: material.clone(),
            },
            size_pixels: 24.0,
            color: Color {
                red: 255,
                green: 255,
                blue: 255,
                alpha: 255,
            },
            background: None,
            outline: None,
            shadow: None,
            fallback_fonts: vec![FontRef::Material {
                material_id: MaterialId::new("med_fallback").unwrap(),
            }],
            spans: vec![TextSpan {
                start: 0,
                end: 1,
                font: Some(FontRef::Material {
                    material_id: MaterialId::new("med_span").unwrap(),
                }),
                font_weight: None,
                font_style: None,
                size_pixels: None,
                color: None,
            }],
            ..TextStyle::default()
        },
    };
    assert_eq!(
        required(&envelope),
        BTreeSet::from([
            material,
            MaterialId::new("med_fallback").unwrap(),
            MaterialId::new("med_span").unwrap(),
        ])
    );
    envelope.project.sequences[1].tracks[0].clips[0].source = ClipSource::Generated {
        generator: Generator::Transparent,
    };
    assert!(required(&envelope).is_empty());
    envelope.project.sequences[0].tracks[0].clips[0].source = ClipSource::Sequence {
        sequence_id: SequenceId::new("seq_main").unwrap(),
    };
    assert!(required(&envelope).is_empty());
    envelope.project.sequences[0].tracks[0].clips[0].source = ClipSource::Sequence {
        sequence_id: SequenceId::new("seq_missing").unwrap(),
    };
    assert!(required(&envelope).is_empty());
}

#[test]
fn collector_reports_an_unknown_config_before_hydration() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let envelope = crate::canonical::load(&project).unwrap();
    let error =
        crate::requirements::collect(&envelope, &RenderConfigId::new("out_missing").unwrap())
            .unwrap_err();
    assert!(error.to_string().contains("RENDER_CONFIG_NOT_FOUND"));
}

fn required(envelope: &veac_ir::ProjectEnvelope) -> BTreeSet<MaterialId> {
    crate::requirements::collect(envelope, &RenderConfigId::new("out_main").unwrap()).unwrap()
}

fn add_material(envelope: &mut veac_ir::ProjectEnvelope, id: &str) {
    let mut material = envelope.project.materials[0].clone();
    material.id = MaterialId::new(id).unwrap();
    envelope.project.materials.push(material);
}

fn media(id: &str) -> ClipSource {
    ClipSource::Media {
        material_id: MaterialId::new(id).unwrap(),
    }
}
