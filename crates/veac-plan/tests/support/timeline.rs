use veac_plan::canonical::*;

use super::{range, time};

pub fn track(id: &str, kind: TrackKind, order: i32, clips: Vec<Clip>) -> Track {
    Track {
        id: TrackId::new(id).expect("track id"),
        kind,
        order,
        placement_mode: PlacementMode::Free,
        state: TrackState {
            enabled: true,
            muted: false,
            solo: false,
            locked: false,
        },
        routing: TrackRouting::Default,
        clips,
    }
}

pub fn media_clip(id: &str, material_id: &str, start: i64) -> Clip {
    Clip {
        id: ItemId::new(id).expect("item id"),
        enabled: true,
        record_range: range(start, 600),
        source: ClipSource::Media {
            material_id: MaterialId::new(material_id).expect("material id"),
        },
        source_mapping: Some(SourceMapping::linear(
            time(0),
            Rational::new(1, 1).expect("rate"),
        )),
        visual: None,
        audio: None,
        effects: Vec::new(),
        replaceable: None,
        template_editable_text: false,
        authorship: None,
    }
}

pub fn generated_clip(id: &str, generator: Generator, start: i64) -> Clip {
    Clip {
        id: ItemId::new(id).expect("item id"),
        enabled: true,
        record_range: range(start, 300),
        source: ClipSource::Generated { generator },
        source_mapping: None,
        visual: None,
        audio: None,
        effects: Vec::new(),
        replaceable: None,
        template_editable_text: false,
        authorship: None,
    }
}

pub fn sequence(id: &str, tracks: Vec<Track>) -> Sequence {
    Sequence {
        id: SequenceId::new(id).expect("sequence id"),
        name: id.to_owned(),
        settings: SequenceSettings {
            width: 1920,
            height: 1080,
            frame_rate: Rational::new(30, 1).expect("frame rate"),
            sample_rate: 48_000,
        },
        tracks,
        applies: Vec::new(),
        authorship: None,
    }
}
