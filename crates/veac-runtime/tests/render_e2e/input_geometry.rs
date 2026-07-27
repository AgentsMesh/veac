use std::process::Command;

use tempfile::tempdir;

use super::support::*;

#[test]
fn sample_aspect_and_display_rotation_render_with_transparent_corners() {
    let temp = tempdir().unwrap();
    let source = rotated_sar_fixture(temp.path());
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_geometry",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_background",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_background", color(20, 180, 40), 0, 1_000)],
        ),
        track(
            "trk_geometry",
            TrackKind::Video,
            1,
            vec![media_clip("itm_geometry", "med_geometry", 0, 1_000)],
        ),
    ]);
    let output = temp.path().join("geometry.mp4");
    let assets = BTreeMap::from([("med_geometry".to_owned(), source)]);
    let rendered = render(canonical, &assets, &output);

    assert_media_contract(&output, 0, 1.0);
    let info = &rendered.plan.inputs[0].video.as_ref().unwrap().info;
    assert_eq!(info.sample_aspect_ratio, Rational::new(2, 1).unwrap());
    assert_eq!(info.rotation_degrees, 45);
    let graph = rendered.command.filter_graph.unwrap();
    assert!(graph.contains("scale=w='iw*2/1':h=ih,setsar=1"), "{graph}");
    assert!(
        graph.contains("format=rgba,rotate=angle='45*PI/180'"),
        "{graph}"
    );
    let center = rgb_at(&output, 0.5, WIDTH / 2, HEIGHT / 2);
    let corners = [
        (2, 2),
        (2, HEIGHT - 3),
        (WIDTH - 3, 2),
        (WIDTH - 3, HEIGHT - 3),
    ]
    .map(|(x, y)| rgb_at(&output, 0.5, x, y));
    assert!(center[0] > 150 && center[1] < 100, "center {center:?}");
    assert!(
        corners
            .iter()
            .any(|pixel| u16::from(pixel[1]) > u16::from(pixel[0]) * 2),
        "transparent corners {corners:?}\n{graph}"
    );
}

fn rotated_sar_fixture(directory: &Path) -> PathBuf {
    let base = directory.join("geometry-base.mp4");
    let encoded = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=64x32:r=10:d=1",
            "-vf",
            "setsar=2/1",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(&base)
        .output()
        .unwrap();
    assert!(encoded.status.success());
    let rotated = directory.join("geometry-rotated.mp4");
    let remuxed = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-display_rotation:v:0",
            "45",
            "-i",
        ])
        .arg(&base)
        .args(["-c", "copy"])
        .arg(&rotated)
        .output()
        .unwrap();
    assert!(remuxed.status.success());
    rotated
}
