use veac_ir::*;

use crate::test_support::{negotiated, output_for, requests};
use crate::*;

#[path = "support/context.rs"]
mod context;
pub(super) use context::*;
#[path = "support/annotation.rs"]
mod annotation;
#[path = "support/executable.rs"]
mod executable;
pub(super) use annotation::*;
#[path = "support/application.rs"]
mod application;
pub(super) use application::*;
#[path = "support/media.rs"]
mod media;
pub(super) use media::*;
#[path = "support/visual.rs"]
mod visual_support;
pub(super) use visual_support::*;

pub(crate) fn project() -> ProjectEnvelope {
    let sequence_id = SequenceId::new("seq_main").unwrap();
    executable::project_envelope(Project {
        id: ProjectId::new("prj_provider").unwrap(),
        revision: 7,
        timebase: 100,
        entry_sequence_id: sequence_id.clone(),
        render_configs: Vec::new(),
        materials: source_materials(),
        multicam_groups: Vec::new(),
        annotations: Vec::new(),
        relations: Vec::new(),
        sequences: vec![Sequence {
            id: sequence_id,
            name: "Main".into(),
            settings: SequenceSettings {
                width: 1920,
                height: 1080,
                frame_rate: Rational::new(30, 1).unwrap(),
                sample_rate: 48_000,
            },
            tracks: vec![
                visual_track(),
                caption_track(),
                audio_track(),
                video_track(),
            ],
            applies: Vec::new(),
            authorship: None,
        }],
        applied_operations: Vec::new(),
        authorship: None,
    })
}

pub(crate) fn exchange(
    capability: Capability,
) -> (ProviderRequestEnvelope, ProviderResponseEnvelope) {
    let payload = requests()
        .into_iter()
        .find(|value| value.capability() == capability)
        .unwrap();
    let request = ProviderRequestEnvelope::new(negotiated(capability), payload).unwrap();
    let response = ProviderResponseEnvelope::new(&request, output_for(&request)).unwrap();
    (request, response)
}

fn visual_track() -> Track {
    track(
        "trk_visual",
        TrackKind::Visual,
        vec![text_clip(), visual_clip()],
    )
}

fn caption_track() -> Track {
    track("trk_captions", TrackKind::Caption, vec![caption_clip()])
}

fn track(id: &str, kind: TrackKind, clips: Vec<Clip>) -> Track {
    Track {
        id: TrackId::new(id).unwrap(),
        kind,
        order: if kind == TrackKind::Visual { 0 } else { 10 },
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

fn text_clip() -> Clip {
    clip(
        "itm_text",
        TimeRange::new(time(0), time(10)).unwrap(),
        ClipSource::Text {
            text: "hello".into(),
            style: TextStyle::default(),
        },
    )
}

fn visual_clip() -> Clip {
    clip(
        "itm_visual",
        TimeRange::new(time(0), time(100)).unwrap(),
        ClipSource::Generated {
            generator: Generator::Transparent,
        },
    )
}

fn caption_clip() -> Clip {
    clip(
        "itm_existing",
        TimeRange::new(time(20), time(10)).unwrap(),
        ClipSource::Caption {
            text: "existing".into(),
            speaker: None,
            cue: Box::default(),
            style: TextStyle::default(),
        },
    )
}

fn clip(id: &str, record_range: TimeRange, source: ClipSource) -> Clip {
    Clip {
        id: ItemId::new(id).unwrap(),
        enabled: true,
        record_range,
        source,
        source_mapping: None,
        visual: Some(visual()),
        audio: None,
        effects: Vec::new(),
        replaceable: None,
        template_editable_text: false,
        authorship: None,
    }
}

pub(super) fn visual() -> VisualProperties {
    let position = Point {
        x: Length {
            value: 0.0,
            unit: LengthUnit::Pixels,
        },
        y: Length {
            value: 0.0,
            unit: LengthUnit::Pixels,
        },
    };
    VisualProperties {
        placement: Placement::Absolute { position },
        frame: None,
        transform: Transform2D {
            position: Animatable::constant(position),
            scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            shear: Vec2 { x: 0.0, y: 0.0 },
            flip_horizontal: false,
            flip_vertical: false,
            rotation_degrees: Animatable::constant(0.0),
            anchor: Vec2 { x: 0.5, y: 0.5 },
            crop: None,
        },
        opacity: Animatable::constant(1.0),
        compositing: Compositing {
            z_index: 0,
            blend_mode: BlendMode::Normal,
        },
        masks: Vec::new(),
        card: None,
        color_pipeline: None,
    }
}

pub(super) fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 100).unwrap()
}
