use std::collections::BTreeSet;
use std::process::Command;

use tempfile::tempdir;
use veac_artifact::ArtifactStore;
use veac_codegen::emitter::BackendAction;

use super::support::*;

#[test]
fn ten_bit_luma_matte_retains_more_than_eight_bit_alpha_precision() {
    let temp = tempdir().unwrap();
    let matte_path = ten_bit_matte(temp.path());
    let mut canonical = project(false);
    configure_alpha_output(&mut canonical);
    canonical.project.materials.push(material(
        "med_matte_precision",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    let mut target = solid_clip("itm_precision_target", color(255, 0, 0), 0, 1_000);
    target.visual = Some(full_visual());
    let mut matte = media_clip("itm_precision_matte", "med_matte_precision", 0, 1_000);
    matte.visual = Some(full_visual());
    canonical.project.sequences[0].tracks.extend([
        track("trk_precision_target", TrackKind::Visual, 0, vec![target]),
        track("trk_precision_matte", TrackKind::Visual, 1, vec![matte]),
    ]);
    add_matte(
        &mut canonical,
        "seq_main",
        "itm_precision_matte",
        "itm_precision_target",
        TrackMatteMode::Luma,
        false,
    );
    let assets = BTreeMap::from([("med_matte_precision".to_owned(), matte_path)]);
    let delivery = prepare_delivery(canonical, &assets, temp.path());
    let graph = delivery
        .bundle
        .tasks()
        .iter()
        .find_map(|task| match &task.action {
            BackendAction::Ffmpeg(command) => command.filter_graph.as_deref(),
            BackendAction::WriteFile { .. } => None,
        })
        .unwrap();
    delivery.execute(&ArtifactStore::new(temp.path().join("store")));

    let alpha = raw_alpha16_frame(delivery.path("dlv_main"));
    let unique: BTreeSet<_> = alpha[..WIDTH as usize].iter().copied().collect();
    assert!(
        unique.len() > 50,
        "matte was quantized: {} levels\n{graph}",
        unique.len(),
    );
}

fn configure_alpha_output(value: &mut ProjectEnvelope) {
    let deliverable = &mut value.project.render_configs[0].deliverables[0];
    deliverable.target = DeliverableTarget::File {
        name: "matte-precision.mov".to_owned(),
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

fn ten_bit_matte(directory: &Path) -> PathBuf {
    let output = directory.join("matte-10bit.mkv");
    let result = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "nullsrc=s=96x54:r=10:d=1,format=yuv444p10le,geq=lum='400+X':cb=512:cr=512",
            "-c:v",
            "ffv1",
            "-level",
            "3",
        ])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    output
}
