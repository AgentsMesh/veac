use std::process::Command;

use tempfile::tempdir;

use super::support::*;

#[test]
fn directional_blur_expands_transparent_pixels_on_the_selected_axis() {
    let temp = tempdir().unwrap();
    let source = transparent_square(temp.path());
    let output = temp.path().join("directional-orientation.mp4");
    render_cases(
        &source,
        &output,
        vec![
            image_clip("baseline", 0, 1_000, None),
            image_clip("zero", 1_000, 1_000, Some(blur(137.0, constant(0.0)))),
            image_clip("horizontal", 2_000, 1_000, Some(blur(0.0, constant(12.0)))),
            image_clip("vertical", 3_000, 1_000, Some(blur(90.0, constant(12.0)))),
        ],
        4_000,
    );

    let baseline_frame = rgb_frame(&output, 0.5);
    let zero_frame = rgb_frame(&output, 1.5);
    let baseline = lit_bounds(&baseline_frame);
    let horizontal = lit_bounds(&rgb_frame(&output, 2.5));
    let vertical = lit_bounds(&rgb_frame(&output, 3.5));
    assert!(
        changed_channels(&baseline_frame, &zero_frame) < 20,
        "zero radius changed pixels"
    );
    assert!(
        horizontal.0 > baseline.0 + 10,
        "{baseline:?} -> {horizontal:?}"
    );
    assert!(
        horizontal.1 <= baseline.1 + 2,
        "{baseline:?} -> {horizontal:?}"
    );
    assert!(vertical.1 > baseline.1 + 10, "{baseline:?} -> {vertical:?}");
    assert!(vertical.0 <= baseline.0 + 2, "{baseline:?} -> {vertical:?}");
}

#[test]
fn animated_directional_blur_changes_radius_during_real_rendering() {
    let temp = tempdir().unwrap();
    let source = transparent_square(temp.path());
    let output = temp.path().join("directional-animation.mp4");
    let radius = Animatable::Keyframes {
        keyframes: vec![
            keyframe("kf_radius_start", 0, 0.0),
            keyframe("kf_radius_end", 1_000, 12.0),
        ],
    };
    render_cases(
        &source,
        &output,
        vec![image_clip("animated", 0, 1_000, Some(blur(0.0, radius)))],
        1_000,
    );

    let early = lit_bounds(&rgb_frame(&output, 0.15));
    let late = lit_bounds(&rgb_frame(&output, 0.85));
    assert!(late.0 > early.0 + 8, "{early:?} -> {late:?}");
    assert!(late.1 <= early.1 + 2, "{early:?} -> {late:?}");
}

#[test]
fn directional_blur_preserves_color_on_semitransparent_edges() {
    let temp = tempdir().unwrap();
    let source = transparent_box(temp.path(), "directional-red.png", "red@0.5");
    let output = temp.path().join("directional-alpha-color.mp4");
    let backgrounds = vec![
        visual_solid_clip("itm_black", color(0, 0, 0), 0, 1_000),
        visual_solid_clip("itm_white", color(255, 255, 255), 1_000, 1_000),
    ];
    let clips = vec![
        image_clip("red-black", 0, 1_000, Some(blur(0.0, constant(12.0)))),
        image_clip("red-white", 1_000, 1_000, Some(blur(0.0, constant(12.0)))),
    ];
    render_cases_with_backgrounds(&source, &output, clips, backgrounds, 2_000);

    let black = rgb_at(&output, 0.5, 38, 27);
    let white = rgb_at(&output, 1.5, 38, 27);
    let alpha = 1.0 - f64::from(white[1].saturating_sub(black[1])) / 255.0;
    let straight_red = f64::from(black[0]) / alpha;
    assert!(
        (0.08..0.8).contains(&alpha),
        "black={black:?}, white={white:?}"
    );
    assert!(straight_red > 210.0, "red={straight_red}, alpha={alpha}");
    assert!(black[0] > black[1] + 15, "black={black:?}");
    assert!(white[0] > white[1] + 15, "white={white:?}");
}

fn render_cases(source: &Path, output: &Path, clips: Vec<Clip>, duration_ms: i64) {
    let backgrounds = vec![visual_solid_clip(
        "itm_directional_bg",
        color(0, 0, 0),
        0,
        duration_ms,
    )];
    render_cases_with_backgrounds(source, output, clips, backgrounds, duration_ms);
}

fn render_cases_with_backgrounds(
    source: &Path,
    output: &Path,
    clips: Vec<Clip>,
    backgrounds: Vec<Clip>,
    duration_ms: i64,
) {
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_directional_square",
        MaterialKind::Image,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    canonical.project.sequences[0].tracks = vec![
        track("trk_directional_bg", TrackKind::Video, 0, backgrounds),
        track("trk_directional_fg", TrackKind::Visual, 1, clips),
    ];
    render(
        canonical,
        &BTreeMap::from([("med_directional_square".to_owned(), source.to_path_buf())]),
        output,
    );
    assert_media_contract(output, 0, duration_ms as f64 / 1_000.0);
}

fn image_clip(id: &str, start: i64, duration: i64, effect: Option<Effect>) -> Clip {
    let mut clip = media_clip(
        &format!("itm_directional_{id}"),
        "med_directional_square",
        start,
        duration,
    );
    clip.visual = Some(full_visual());
    clip.effects
        .extend(effect.map(|value| video_effect(&format!("fx_{id}"), value)));
    clip
}

fn blur(angle: f64, radius: Animatable<f64>) -> Effect {
    Effect::VideoDirectionalBlur {
        angle_degrees: constant(angle),
        radius,
    }
}

fn constant(value: f64) -> Animatable<f64> {
    Animatable::constant(value)
}

fn keyframe(id: &str, milliseconds: i64, value: f64) -> Keyframe<f64> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(milliseconds),
        value,
        interpolation: Interpolation::Linear,
    }
}

fn transparent_square(directory: &Path) -> PathBuf {
    transparent_box(directory, "directional-square.png", "white@1")
}

fn transparent_box(directory: &Path, name: &str, fill: &str) -> PathBuf {
    let output = directory.join(name);
    run(Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
        ])
        .arg(format!(
            "color=c=black@0:s={WIDTH}x{HEIGHT},format=rgba,\
             drawbox=x=44:y=23:w=8:h=8:color={fill}:t=fill:replace=1"
        ))
        .args(["-frames:v", "1", "-update", "1"])
        .arg(&output));
    output
}

fn run(command: &mut Command) {
    let output = command.output().expect("run FFmpeg fixture command");
    assert!(
        output.status.success(),
        "fixture generation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
