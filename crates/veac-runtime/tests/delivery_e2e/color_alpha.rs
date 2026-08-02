use std::process::Command;

use tempfile::tempdir;
use veac_artifact::ArtifactStore;

use super::support::*;

#[test]
fn color_conversion_and_multiply_preserve_real_source_alpha() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    configure_alpha_output(&mut canonical);
    let mut clip = solid_clip(
        "itm_alpha_color",
        Color {
            red: 255,
            green: 0,
            blue: 0,
            alpha: 128,
        },
        0,
        1_000,
    );
    let mut visual = full_visual();
    visual.compositing.blend_mode = BlendMode::Multiply;
    visual.color_pipeline = Some(ColorPipeline {
        input: rec709(),
        working: ColorSpace {
            primaries: ColorPrimaries::Bt2020,
            transfer: ColorTransfer::Smpte2084,
            matrix: ColorMatrix::Bt2020Ncl,
            range: ColorRange::Limited,
        },
        output: rec709(),
        stages: vec![],
    });
    clip.visual = Some(visual);
    canonical.project.sequences[0].tracks.push(track(
        "trk_alpha_color",
        TrackKind::Visual,
        0,
        vec![clip],
    ));

    let delivery = prepare_delivery(canonical, &BTreeMap::new(), temp.path());
    delivery.execute(&ArtifactStore::new(temp.path().join("store")));
    let alpha = raw_alpha(delivery.path("dlv_main"));
    let rgba = raw_rgba_frame(delivery.path("dlv_main"));

    assert_eq!(alpha.len(), (WIDTH * HEIGHT) as usize);
    assert!(alpha.iter().all(|value| (110..=145).contains(value)));
    assert!(rgba.chunks_exact(4).all(|pixel| {
        pixel[0] > 180 && pixel[1] < 60 && pixel[2] < 60 && (110..=145).contains(&pixel[3])
    }));
}

#[test]
fn color_stabilize_and_key_effects_preserve_or_multiply_real_alpha() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    configure_alpha_output(&mut canonical);
    let mut clip = solid_clip(
        "itm_effect_alpha",
        Color {
            red: 255,
            green: 0,
            blue: 0,
            alpha: 128,
        },
        0,
        1_000,
    );
    clip.visual = Some(full_visual());
    clip.effects = vec![
        video_effect(
            "fx_alpha_vignette",
            "video.vignette",
            BTreeMap::from([("amount".to_owned(), ParameterValue::Number { value: 0.5 })]),
        ),
        video_effect(
            "fx_alpha_stabilize",
            "video.stabilize",
            BTreeMap::from([(
                "enabled".to_owned(),
                ParameterValue::Boolean { value: true },
            )]),
        ),
        video_effect(
            "fx_alpha_key",
            "video.chroma_key",
            BTreeMap::from([
                (
                    "color".to_owned(),
                    ParameterValue::Color {
                        value: color(0, 255, 0),
                    },
                ),
                (
                    "similarity".to_owned(),
                    ParameterValue::Number { value: 0.1 },
                ),
                ("blend".to_owned(), ParameterValue::Number { value: 0.0 }),
            ]),
        ),
    ];
    canonical.project.sequences[0].tracks.push(track(
        "trk_effect_alpha",
        TrackKind::Visual,
        0,
        vec![clip],
    ));

    let delivery = prepare_delivery(canonical, &BTreeMap::new(), temp.path());
    delivery.execute(&ArtifactStore::new(temp.path().join("store")));
    let alpha = raw_alpha(delivery.path("dlv_main"));

    assert!(alpha.iter().all(|value| (110..=145).contains(value)));
}

fn configure_alpha_output(value: &mut ProjectEnvelope) {
    let deliverable = &mut value.project.render_configs[0].deliverables[0];
    deliverable.target = DeliverableTarget::File {
        name: "color-alpha.mov".to_owned(),
    };
    let DeliverableKind::Video(settings) = &mut deliverable.kind else {
        panic!("video fixture")
    };
    settings.container = OutputFormat::Mov;
    settings.audio = None;
    settings.video = VideoOutput {
        codec: VideoCodec::ProRes,
        pixel_format: PixelFormat::Yuva444p10le,
        alpha: AlphaMode::Straight,
        color_space: None,
        rate_control: VideoRateControl::Lossless,
        gop_size: None,
        b_frames: None,
        profile: Some(VideoProfile::ProRes4444),
        level: None,
    };
}

fn raw_alpha(path: &Path) -> Vec<u8> {
    let output = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(path)
        .args([
            "-vf",
            "alphaextract",
            "-frames:v",
            "1",
            "-pix_fmt",
            "gray",
            "-f",
            "rawvideo",
            "-",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn rec709() -> ColorSpace {
    ColorSpace {
        primaries: ColorPrimaries::Bt709,
        transfer: ColorTransfer::Bt709,
        matrix: ColorMatrix::Bt709,
        range: ColorRange::Limited,
    }
}
