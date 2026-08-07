use veac_ir::{
    AudioCodec, ClipSource, FontRef, HashAlgorithm, Material, MaterialKind, MaterialSource,
    SourceTimeMap, StreamChoice, TrackKind,
};

use super::support;

#[path = "media_caption/source.rs"]
mod source;
use source::SOURCE;

#[test]
fn resources_lower_with_exact_kind_identity_and_stream_intent() {
    let envelope = support::envelope(SOURCE);
    let materials = &envelope.project.materials;
    assert_eq!(materials.len(), 3);
    assert_material(
        find(materials, MaterialKind::Image),
        "assets/poster.png",
        'a',
        StreamChoice::Auto,
        StreamChoice::Disabled,
    );
    assert_material(
        find(materials, MaterialKind::Audio),
        "assets/voice.wav",
        'b',
        StreamChoice::Disabled,
        StreamChoice::Auto,
    );
    assert_material(
        find(materials, MaterialKind::Font),
        "assets/caption.ttf",
        'c',
        StreamChoice::Disabled,
        StreamChoice::Disabled,
    );
}

#[test]
fn audio_and_caption_items_lower_to_their_canonical_track_contracts() {
    let envelope = support::envelope(SOURCE);
    let project = &envelope.project;
    let tracks = &project.sequences[0].tracks;
    assert_eq!(
        tracks.iter().map(|track| track.kind).collect::<Vec<_>>(),
        vec![TrackKind::Visual, TrackKind::Audio, TrackKind::Caption]
    );
    assert_media_clip(
        &tracks[0].clips[0],
        find(&project.materials, MaterialKind::Image),
    );
    let audio = &tracks[1].clips[0];
    assert_media_clip(audio, find(&project.materials, MaterialKind::Audio));
    assert!(audio.visual.is_none() && audio.audio.is_none());

    let font = find(&project.materials, MaterialKind::Font);
    let plain = &tracks[2].clips[0];
    let spoken = &tracks[2].clips[1];
    assert_caption(plain, "开场字幕", None, font);
    assert_caption(spoken, "开始创作", Some("旁白"), font);
    assert!(plain.visual.is_some() && spoken.visual.is_some());
    assert!(plain.audio.is_none() && spoken.audio.is_none());
}

#[test]
fn explicit_delivery_publishes_burned_in_caption_and_aac_audio() {
    let envelope = support::envelope(SOURCE);
    let output = &envelope.project.render_configs[0];
    assert_eq!(
        output.raster.as_ref().unwrap().captions,
        veac_ir::CaptionOutput::BurnIn
    );
    let veac_ir::DeliverableKind::Video(video) = &output.deliverables[0].kind else {
        panic!("expected video deliverable")
    };
    let audio = video.audio.as_ref().expect("default audio delivery");
    assert_eq!(audio.codec, AudioCodec::Aac);
    assert_eq!((audio.sample_rate, audio.channels), (48_000, 2));
    veac_ir::validate(&envelope).unwrap();
    let json = veac_ir::canonical_json(&envelope).unwrap();
    assert_eq!(veac_ir::decode_canonical_json(&json).unwrap(), envelope);
}

fn find(materials: &[Material], kind: MaterialKind) -> &Material {
    materials.iter().find(|value| value.kind == kind).unwrap()
}

fn assert_material(
    material: &Material,
    uri: &str,
    digest: char,
    video: StreamChoice,
    audio: StreamChoice,
) {
    assert_eq!(
        material.source,
        MaterialSource::File {
            uri: uri.to_owned()
        }
    );
    let identity = material.identity.as_ref().expect("authored identity");
    assert_eq!(identity.algorithm, HashAlgorithm::Sha256);
    assert_eq!(identity.digest, digest.to_string().repeat(64));
    assert_eq!(
        (material.stream_intent.video, material.stream_intent.audio),
        (video, audio)
    );
    assert!(material.probe.is_none());
}

fn assert_media_clip(clip: &veac_ir::Clip, material: &Material) {
    assert_eq!(
        clip.source,
        ClipSource::Media {
            material_id: material.id.clone()
        }
    );
    let mapping = clip.source_mapping.as_ref().expect("linear media mapping");
    assert!(matches!(mapping.time_map, SourceTimeMap::Linear { .. }));
}

fn assert_caption(
    clip: &veac_ir::Clip,
    expected_text: &str,
    expected_speaker: Option<&str>,
    font: &Material,
) {
    let ClipSource::Caption {
        text,
        speaker,
        style,
        ..
    } = &clip.source
    else {
        panic!("expected caption source")
    };
    assert_eq!(text, expected_text);
    assert_eq!(speaker.as_deref(), expected_speaker);
    assert_eq!(
        style.font,
        FontRef::Material {
            material_id: font.id.clone()
        }
    );
    assert_eq!(
        (style.color.red, style.color.green, style.color.blue),
        (254, 243, 199)
    );
    assert_eq!(style.size_pixels, 32.0);
}
