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

#[test]
fn progress_angle_binding_turns_a_fixed_radius_directional_blur() {
    let temp = tempdir().unwrap();
    let source = directional_target(temp.path());
    let output = temp.path().join("temporal-directional-angle.mp4");
    let mut project = project(false);
    project.project.materials.push(material(
        "med_temporal_directional",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    let binding = install(
        &mut project,
        "directional_angle",
        "itm_temporal_directional",
        TemporalType::Scalar,
        scaled_progress(90.0),
        2,
    );
    let mut clip = media_clip(
        "itm_temporal_directional",
        "med_temporal_directional",
        0,
        1_000,
    );
    clip.visual = Some(full_visual());
    clip.effects.push(video_effect(
        "fx_temporal_directional",
        Effect::VideoDirectionalBlur {
            angle_degrees: Animatable::Binding {
                binding_id: binding,
            },
            radius: Animatable::constant(12.0),
        },
    ));
    project.project.sequences[0].tracks.push(track(
        "trk_temporal_directional",
        TrackKind::Video,
        0,
        vec![clip],
    ));
    let rendered = render(
        project,
        &BTreeMap::from([("med_temporal_directional".to_owned(), source)]),
        &output,
    );

    let graph = rendered.command.filter_graph.unwrap();
    assert!(
        graph.contains(":radius=12") && graph.contains(" angle "),
        "{graph}"
    );
    let early = lit_bounds(&rgb_frame(&output, 0.05));
    let late = lit_bounds(&rgb_frame(&output, 0.85));
    assert!(early.0 > early.1 + 8, "early={early:?}");
    assert!(late.1 > late.0 + 5, "late={late:?}");
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

fn directional_target(directory: &Path) -> PathBuf {
    let output = directory.join("temporal-directional-target.mp4");
    run(Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            &format!(
                "color=c=black:s={WIDTH}x{HEIGHT}:r={FPS}:d=1,\
                 drawbox=x=44:y=23:w=8:h=8:color=white:t=fill"
            ),
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
