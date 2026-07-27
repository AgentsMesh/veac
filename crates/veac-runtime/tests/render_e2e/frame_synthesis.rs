use std::process::Command;

use tempfile::tempdir;

use super::support::*;

#[test]
fn blend_and_motion_compensation_create_observable_interframes() {
    let temp = tempdir().unwrap();
    let source = motion_fixture(temp.path());
    let nearest = temp.path().join("nearest.mp4");
    let blend = temp.path().join("blend.mp4");
    let motion = temp.path().join("motion.mp4");

    let nearest_render = render_policy(&source, &nearest, FrameSynthesisPolicy::Nearest);
    let blend_render = render_policy(&source, &blend, FrameSynthesisPolicy::Blend);
    let motion_render = render_policy(&source, &motion, FrameSynthesisPolicy::MotionCompensated);
    assert_media_contract(&blend, 0, 1.5);
    assert_media_contract(&motion, 0, 1.5);
    let blend_difference = max_frame_difference(&nearest, &blend);
    let motion_difference = max_frame_difference(&nearest, &motion);
    assert!(
        blend_difference > 2.0,
        "blend difference {blend_difference}"
    );
    assert!(
        motion_difference > 2.0,
        "motion difference {motion_difference}"
    );
    assert_graph(nearest_render, "fps=10/1");
    assert_graph(blend_render, "mi_mode=blend");
    assert_graph(motion_render, "mi_mode=mci");
}

fn render_policy(source: &Path, output: &Path, policy: FrameSynthesisPolicy) -> Rendered {
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_motion",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    let mut clip = media_clip("itm_motion", "med_motion", 0, 1_500);
    clip.source_mapping = Some(linear_mapping(1, ratio(1, 1), policy));
    canonical.project.sequences[0].tracks.push(track(
        "trk_motion",
        TrackKind::Video,
        0,
        vec![clip],
    ));
    render(
        canonical,
        &BTreeMap::from([("med_motion".to_owned(), source.to_path_buf())]),
        output,
    )
}

fn motion_fixture(directory: &Path) -> PathBuf {
    let output = directory.join("motion-source.mp4");
    let result = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=96x54:rate=5:duration=2",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(&output)
        .output()
        .unwrap();
    assert!(result.status.success(), "FFmpeg motion fixture failed");
    output
}

fn max_frame_difference(left: &Path, right: &Path) -> f64 {
    [0.3, 0.5, 0.7, 0.9, 1.1]
        .into_iter()
        .map(|second| {
            let left = rgb_frame(left, second);
            let right = rgb_frame(right, second);
            left.iter()
                .zip(right)
                .map(|(left, right)| f64::from(left.abs_diff(right)))
                .sum::<f64>()
                / left.len() as f64
        })
        .fold(0.0, f64::max)
}

fn assert_graph(rendered: Rendered, needle: &str) {
    assert!(rendered.command.filter_graph.unwrap().contains(needle));
}
