use tempfile::tempdir;
use veac_artifact::ArtifactStore;

use super::support::*;

#[test]
fn half_alpha_blue_over_half_alpha_red_uses_straight_source_over() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    configure_alpha_output(&mut canonical);
    let mut red = solid_clip("itm_half_red", rgba(255, 0, 0, 128), 0, 1_000);
    red.visual = Some(full_visual());
    let mut blue = solid_clip("itm_half_blue", rgba(0, 0, 255, 128), 0, 1_000);
    blue.visual = Some(full_visual());
    canonical.project.sequences[0].tracks.extend([
        track("trk_half_red", TrackKind::Video, 0, vec![red]),
        track("trk_half_blue", TrackKind::Visual, 1, vec![blue]),
    ]);

    let delivery = prepare_delivery(canonical, &BTreeMap::new(), temp.path());
    delivery.execute(&ArtifactStore::new(temp.path().join("store")));
    let frame = raw_rgba_frame(delivery.path("dlv_main"));
    let center = pixel(&frame, WIDTH / 2, HEIGHT / 2);

    assert!((180..=202).contains(&center[3]), "alpha={center:?}");
    assert!((55..=115).contains(&center[0]), "red={center:?}");
    assert!((140..=210).contains(&center[2]), "blue={center:?}");
    assert!(center[2] > center[0] + 60, "straight RGB={center:?}");
}

fn configure_alpha_output(value: &mut ProjectEnvelope) {
    let deliverable = &mut value.project.render_configs[0].deliverables[0];
    deliverable.target = DeliverableTarget::File {
        name: "source-over.mov".to_owned(),
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

fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Color {
    Color {
        red,
        green,
        blue,
        alpha,
    }
}

fn pixel(frame: &[u8], x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * WIDTH + x) * 4) as usize;
    frame[offset..offset + 4].try_into().unwrap()
}
