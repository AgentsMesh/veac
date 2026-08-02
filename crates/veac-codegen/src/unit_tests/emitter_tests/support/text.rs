use base64::Engine as _;
use veac_plan::canonical::*;

use super::{file_identity, fixture, test_font_path, time, visual};

pub fn text_fixture(caption: bool) -> ProjectEnvelope {
    let mut project = fixture();
    project.project.materials.push(Material {
        id: MaterialId::new("med_font").unwrap(),
        kind: MaterialKind::Font,
        source: MaterialSource::File {
            uri: "font.ttf".to_owned(),
        },
        identity: Some(file_identity(&test_font_path())),
        stream_intent: StreamIntent {
            video: StreamChoice::Disabled,
            audio: StreamChoice::Disabled,
        },
        probe: None,
        metadata: Default::default(),
    });
    let source = if caption {
        ClipSource::Caption {
            text: escaped_text(),
            speaker: None,
            style: style(),
        }
    } else {
        ClipSource::Text {
            text: escaped_text(),
            style: style(),
        }
    };
    let mut clip = project.project.sequences[0].tracks[0].clips[0].clone();
    clip.id = ItemId::new(if caption {
        "itm_caption"
    } else {
        "itm_text_full"
    })
    .unwrap();
    clip.record_range.start = time(600);
    clip.source = source;
    clip.source_mapping = None;
    clip.audio = None;
    let mut properties = visual();
    properties.frame = None;
    clip.visual = Some(properties);
    project.project.sequences[0].tracks.push(Track {
        id: TrackId::new(if caption {
            "trk_caption"
        } else {
            "trk_text_full"
        })
        .unwrap(),
        kind: if caption {
            TrackKind::Caption
        } else {
            TrackKind::Visual
        },
        order: 10,
        placement_mode: PlacementMode::Free,
        state: TrackState {
            enabled: true,
            muted: false,
            solo: false,
            locked: false,
        },
        routing: TrackRouting::Default,
        clips: vec![clip],
    });
    project
}

fn style() -> TextStyle {
    TextStyle {
        font: FontRef::Material {
            material_id: MaterialId::new("med_font").unwrap(),
        },
        size_pixels: 28.0,
        color: color(255, 240, 20, 255),
        background: Some(TextBackground {
            color: color(10, 20, 30, 128),
            padding_pixels: 3.0,
        }),
        outline: Some(TextOutline {
            color: color(250, 250, 250, 255),
            width_pixels: 1.5,
        }),
        shadow: Some(Shadow {
            blur_pixels: 2.0,
            opacity: 0.6,
            offset: Vec2 { x: 2.0, y: 3.0 },
            color: color(0, 0, 0, 255),
        }),
        ..TextStyle::default()
    }
}

fn color(red: u8, green: u8, blue: u8, alpha: u8) -> Color {
    Color {
        red,
        green,
        blue,
        alpha,
    }
}

fn escaped_text() -> String {
    "a'b:c%d\\e\nf".to_owned()
}

pub fn ass_script(graph: &str) -> String {
    let marker = "base64\\,";
    let start = graph.find(marker).expect("ASS data URI") + marker.len();
    let encoded = &graph[start..];
    let end = encoded.find('\'').expect("end of ASS data URI");
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&encoded[..end])
        .expect("base64 ASS payload");
    String::from_utf8(bytes).expect("UTF-8 ASS payload")
}
