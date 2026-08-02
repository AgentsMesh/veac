use std::process::Command;

use tempfile::tempdir;

use super::support::*;

#[test]
fn structured_export_settings_reach_ffmpeg_and_the_output_bitstream() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    let output = &mut canonical.project.render_configs[0];
    let deliverable_id = output.deliverables[0].id.clone();
    let delivery = output.video_deliverable_mut(&deliverable_id).unwrap();
    delivery.video = VideoOutput {
        codec: VideoCodec::H264,
        pixel_format: PixelFormat::Yuv420p,
        alpha: AlphaMode::Opaque,
        color_space: None,
        rate_control: VideoRateControl::Crf { value: 18 },
        gop_size: Some(10),
        b_frames: Some(2),
        profile: Some(VideoProfile::H264High),
        level: Some("3.1".to_owned()),
    };
    delivery.optimize_for_streaming = true;
    canonical.project.sequences[0].tracks.push(track(
        "trk_base",
        TrackKind::Video,
        0,
        vec![solid_clip("itm_base", color(20, 80, 160), 0, 1_000)],
    ));
    let path = temp.path().join("export.mp4");
    let rendered = render(canonical, &BTreeMap::new(), &path);
    for (name, value) in [
        ("-crf", "18"),
        ("-g", "10"),
        ("-bf", "2"),
        ("-profile:v", "high"),
        ("-level:v", "3.1"),
        ("-movflags", "+faststart"),
    ] {
        assert!(pair(&rendered.command.output_args, name, value));
    }
    let probe = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=profile,pix_fmt,level",
            "-of",
            "json",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert!(probe.status.success());
    let value: serde_json::Value = serde_json::from_slice(&probe.stdout).unwrap();
    let stream = &value["streams"][0];
    assert_profile(&stream["profile"], &["High", "100"]);
    assert_eq!(stream["pix_fmt"], "yuv420p");
    assert_eq!(stream["level"], 31);
    let bytes = std::fs::read(path).unwrap();
    assert!(atom(&bytes, b"moov") < atom(&bytes, b"mdat"));
}

fn pair(args: &[String], name: &str, value: &str) -> bool {
    args.windows(2).any(|pair| pair == [name, value])
}

fn atom(bytes: &[u8], name: &[u8; 4]) -> usize {
    bytes
        .windows(name.len())
        .position(|window| window == name)
        .expect("MP4 atom")
}
