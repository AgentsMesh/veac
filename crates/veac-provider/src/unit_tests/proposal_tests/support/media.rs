use veac_ir::*;

use crate::{InputArtifact, MaterialInsertion, MediaType, ProviderArtifact};

use super::{time, visual};

#[path = "media/probe.rs"]
mod probe_support;
use probe_support::{intent, probe};

pub(crate) fn source_materials() -> Vec<Material> {
    vec![
        source_material("med_source_audio", MediaType::Audio),
        source_material("med_source_video", MediaType::Video),
    ]
}

pub(crate) fn audio_track() -> Track {
    media_track(
        "trk_audio",
        TrackKind::Audio,
        media_clip("itm_audio", "med_source_audio", false),
    )
}

pub(crate) fn video_track() -> Track {
    media_track(
        "trk_video",
        TrackKind::Video,
        media_clip("itm_video", "med_source_video", true),
    )
}

pub(crate) fn output_material(
    artifact: &ProviderArtifact,
    id: &str,
    kind: MaterialKind,
    duration: RationalTime,
) -> MaterialInsertion {
    let identity = MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: artifact.record.content.value.clone(),
    };
    MaterialInsertion {
        material: Material {
            id: MaterialId::new(id).unwrap(),
            kind,
            source: MaterialSource::File {
                uri: format!("artifacts/{}.bin", artifact.record.key.value),
            },
            identity: Some(identity.clone()),
            stream_intent: intent(kind),
            probe: Some(probe(kind, identity, duration)),
            authorship: None,
        },
        before_id: None,
        after_id: None,
    }
}

pub(crate) fn audio_properties() -> AudioProperties {
    AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: Vec::new(),
        crossfade: None,
    }
}

fn source_material(id: &str, media_type: MediaType) -> Material {
    let input = crate::test_support::input(media_type);
    let kind = match media_type {
        MediaType::Audio => MaterialKind::Audio,
        MediaType::Video => MaterialKind::Video,
        _ => unreachable!(),
    };
    let identity = identity(&input);
    Material {
        id: MaterialId::new(id).unwrap(),
        kind,
        source: MaterialSource::File {
            uri: format!("fixtures/{id}.bin"),
        },
        identity: Some(identity.clone()),
        stream_intent: intent(kind),
        probe: Some(probe(kind, identity, time(100))),
        authorship: None,
    }
}

fn identity(input: &InputArtifact) -> MediaIdentity {
    MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: input.content.value.clone(),
    }
}

fn media_track(id: &str, kind: TrackKind, clip: Clip) -> Track {
    Track {
        id: TrackId::new(id).unwrap(),
        kind,
        order: if kind == TrackKind::Audio { 20 } else { -10 },
        placement_mode: PlacementMode::Free,
        state: TrackState {
            enabled: true,
            muted: false,
            solo: false,
            locked: false,
        },
        routing: TrackRouting::Default,
        clips: vec![clip],
    }
}

fn media_clip(id: &str, material_id: &str, has_visual: bool) -> Clip {
    Clip {
        id: ItemId::new(id).unwrap(),
        enabled: true,
        record_range: TimeRange::new(time(0), time(100)).unwrap(),
        source: ClipSource::Media {
            material_id: MaterialId::new(material_id).unwrap(),
        },
        source_mapping: Some(SourceMapping::linear(time(0), Rational::new(1, 1).unwrap())),
        visual: has_visual.then(visual),
        audio: (!has_visual).then(audio_properties),
        effects: Vec::new(),
        replaceable: None,
        template_editable_text: false,
        authorship: None,
    }
}
