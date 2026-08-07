use tempfile::tempdir;
use veac_artifact::ArtifactStore;
use veac_ir::StreamChoice;

use super::support::*;

mod ass_probe;

#[test]
fn srt_and_webvtt_sidecars_preserve_real_cues_and_timing() {
    let temp = tempdir().unwrap();
    let font = font_fixture();
    let mut canonical = project(false);
    canonical.project.render_configs[0].raster = None;
    canonical.project.materials.push(material(
        "med_caption_font",
        MaterialKind::Font,
        StreamChoice::Disabled,
        StreamChoice::Disabled,
    ));
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_picture",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_picture", color(20, 40, 80), 0, 2_000)],
        ),
        track(
            "trk_captions_plain",
            TrackKind::Caption,
            1,
            vec![
                caption("itm_plain_one", "First line", None, false, 0, 750),
                caption("itm_plain_two", "Second line", None, false, 750, 1_250),
            ],
        ),
        track(
            "trk_captions_ass",
            TrackKind::Caption,
            2,
            vec![
                caption(
                    "itm_caption_one",
                    "First line",
                    Some("Narrator"),
                    true,
                    0,
                    750,
                ),
                caption("itm_caption_two", "Second line", None, false, 750, 1_250),
            ],
        ),
    ]);
    canonical.project.render_configs[0].deliverables = vec![
        sidecar(
            "dlv_ass",
            "captions.ass",
            CaptionSidecarFormat::Ass,
            "trk_captions_ass",
        ),
        sidecar(
            "dlv_srt",
            "captions.srt",
            CaptionSidecarFormat::Srt,
            "trk_captions_plain",
        ),
        sidecar(
            "dlv_vtt",
            "captions.vtt",
            CaptionSidecarFormat::WebVtt,
            "trk_captions_plain",
        ),
    ];
    let assets = BTreeMap::from([("med_caption_font".to_owned(), font)]);
    let delivery = prepare_delivery(canonical, &assets, temp.path());
    let store = ArtifactStore::new(temp.path().join("store"));
    let execution = delivery.execute(&store);
    assert!(execution.tasks.iter().all(|task| !task.cache_hit));
    assert_eq!(
        std::fs::read_to_string(delivery.path("dlv_srt")).unwrap(),
        "1\n00:00:00,000 --> 00:00:00,750\nFirst line\n\n\
         2\n00:00:00,750 --> 00:00:02,000\nSecond line\n\n"
    );
    let ass = std::fs::read_to_string(delivery.path("dlv_ass")).unwrap();
    assert!(ass.contains("[Script Info]\nScriptType: v4.00+"));
    assert!(ass.contains("[V4+ Styles]"));
    assert!(ass.contains("[Events]"));
    assert_eq!(
        ass.lines()
            .filter(|line| line.starts_with("Style: VEAC"))
            .count(),
        1
    );
    let style = ass
        .lines()
        .find(|line| line.starts_with("Style: VEAC0001,"))
        .unwrap();
    assert!(style.contains(",18,&H0028C8F0"));
    assert!(ass.contains(",VEAC0001,Narrator,0,0,0,,"));
    assert!(ass.contains("\\b700\\fs24\\1c&H1414FF&\\1a&H00&"));
    assert!(ass.contains(&format!("\\pos({},{})", WIDTH - 12, HEIGHT - 16)));
    assert!(ass.contains("\\blur1.5\\xshad2\\yshad3"));
    assert!(ass.contains("Dialogue: 0,0:00:00.00,0:00:00.75"));
    assert!(ass.contains("Dialogue: 1,0:00:00.75,0:00:02.00"));
    assert_eq!(ass_probe::codec(delivery.path("dlv_ass")), "ass");
    assert_ne!(
        ass_probe::frame_digest(None, WIDTH, HEIGHT),
        ass_probe::frame_digest(Some(delivery.path("dlv_ass")), WIDTH, HEIGHT)
    );
    assert_eq!(
        std::fs::read_to_string(delivery.path("dlv_vtt")).unwrap(),
        "WEBVTT\n\n00:00:00.000 --> 00:00:00.750\nFirst line\n\n\
         00:00:00.750 --> 00:00:02.000\nSecond line\n\n"
    );
    assert!(delivery
        .execute(&store)
        .tasks
        .iter()
        .all(|task| task.cache_hit));
}

fn caption(
    id: &str,
    text: &str,
    speaker: Option<&str>,
    rich: bool,
    start: i64,
    duration: i64,
) -> Clip {
    let mut style = caption_style();
    if rich {
        style.spans.push(TextSpan {
            start: 0,
            end: 5,
            font: None,
            font_weight: Some(FontWeight::Bold),
            font_style: None,
            size_pixels: Some(24.0),
            color: Some(color(255, 20, 20)),
        });
    }
    let mut clip = text_clip(id, text, style.clone(), start, duration);
    clip.source = ClipSource::Caption {
        text: text.to_owned(),
        speaker: speaker.map(str::to_owned),
        cue: Box::default(),
        style,
    };
    let mut visual = full_visual();
    visual.transform.position = Animatable::constant(Point {
        x: pixels(-12.0),
        y: pixels(-16.0),
    });
    clip.visual = Some(visual);
    clip
}

fn caption_style() -> TextStyle {
    TextStyle {
        font: FontRef::Material {
            material_id: MaterialId::new("med_caption_font").unwrap(),
        },
        size_pixels: 18.0,
        color: color(240, 200, 40),
        font_weight: FontWeight::SemiBold,
        font_style: FontStyle::Italic,
        tracking_pixels: 1.0,
        layout: TextLayout {
            horizontal_alignment: HorizontalTextAlignment::Right,
            vertical_alignment: VerticalTextAlignment::Bottom,
            ..TextLayout::default()
        },
        outline: Some(TextOutline {
            color: color(20, 30, 40),
            width_pixels: 2.0,
        }),
        shadow: Some(Shadow {
            blur_pixels: 1.5,
            opacity: 0.75,
            offset: Vec2 { x: 2.0, y: 3.0 },
            color: color(0, 0, 0),
        }),
        ..TextStyle::default()
    }
}

fn sidecar(id: &str, file: &str, format: CaptionSidecarFormat, track: &str) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        target: DeliverableTarget::File {
            name: file.to_owned(),
        },
        kind: DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
            format,
            track_ids: vec![TrackId::new(track).unwrap()],
        }),
    }
}
