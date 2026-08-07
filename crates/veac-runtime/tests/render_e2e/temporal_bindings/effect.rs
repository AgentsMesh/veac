use std::process::Command;

use tempfile::tempdir;

use super::super::support::*;
use super::{install, scaled_progress};

#[test]
fn progress_binding_drives_a_runtime_effect_parameter() {
    let temp = tempdir().unwrap();
    let source = static_grid(temp.path());
    let output = temp.path().join("temporal-effect.mp4");
    let mut project = project(false);
    project.project.materials.push(material(
        "med_temporal_grid",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    let binding = install(
        &mut project,
        "effect_blur",
        "itm_temporal_effect",
        TemporalType::Scalar,
        scaled_progress(10.0),
        2,
    );
    let mut clip = media_clip("itm_temporal_effect", "med_temporal_grid", 0, 1_000);
    clip.visual = Some(full_visual());
    clip.effects.push(video_effect(
        "fx_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        Effect::VideoBlur {
            radius: Animatable::Binding {
                binding_id: binding,
            },
        },
    ));
    project.project.sequences[0].tracks.push(track(
        "trk_temporal_effect",
        TrackKind::Video,
        0,
        vec![clip],
    ));
    render(
        project,
        &BTreeMap::from([("med_temporal_grid".to_owned(), source)]),
        &output,
    );

    let early = spatial_detail(&rgb_frame(&output, 0.1));
    let late = spatial_detail(&rgb_frame(&output, 0.8));
    assert!(late < early * 0.7, "blur detail {early}..{late}");
}

fn static_grid(directory: &Path) -> PathBuf {
    let output = directory.join("temporal-grid.mp4");
    run(Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            &format!("color=c=white:s={WIDTH}x{HEIGHT}:r={FPS}:d=1,drawgrid=w=6:h=6:t=2:c=black"),
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(&output));
    output
}

fn spatial_detail(frame: &[u8]) -> f64 {
    let mut total = 0.0;
    for y in 0..HEIGHT as usize {
        for x in 1..WIDTH as usize {
            let left = (y * WIDTH as usize + x - 1) * 3;
            let right = left + 3;
            total += (f64::from(frame[left]) - f64::from(frame[right])).abs();
        }
    }
    total
}
