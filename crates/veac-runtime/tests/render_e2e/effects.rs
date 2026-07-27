use std::process::Command;

use tempfile::tempdir;

use super::support::*;

#[test]
fn animated_blur_sharpen_vignette_and_grain_change_real_frames() {
    let temp = tempdir().unwrap();
    let source = pattern_fixture(temp.path());
    let output = temp.path().join("animated-effects.mp4");
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_effect_pattern",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    let cases = [
        ("blur", "video.blur", "radius", 10.0),
        ("sharpen", "video.sharpen", "amount", 10.0),
        ("vignette", "video.vignette", "amount", 1.0),
        ("grain", "video.grain", "amount", 1.0),
    ];
    let mut clips = vec![effect_clip("baseline", 0, None)];
    clips.extend(
        cases
            .into_iter()
            .enumerate()
            .map(|(index, (name, kind, parameter, end))| {
                effect_clip(
                    name,
                    (index as i64 + 1) * 1_000,
                    Some(video_effect(
                        &format!("fx_{name}"),
                        kind,
                        BTreeMap::from([(parameter.to_owned(), curve(name, 0.00001, end))]),
                    )),
                )
            }),
    );
    canonical.project.sequences[0]
        .tracks
        .push(track("trk_effects", TrackKind::Video, 0, clips));
    render(
        canonical,
        &BTreeMap::from([("med_effect_pattern".to_owned(), source)]),
        &output,
    );
    assert_media_contract(&output, 0, 5.0);
    let baseline = rgb_frame(&output, 0.8);
    for (name, second, minimum) in [
        ("blur", 1.8, 500),
        ("sharpen", 2.8, 100),
        ("vignette", 3.8, 300),
        ("grain", 4.8, 500),
    ] {
        let changed = changed_channels(&baseline, &rgb_frame(&output, second));
        assert!(changed > minimum, "{name} changed channels={changed}");
    }
}

#[test]
fn stabilize_changes_observable_frames_from_a_shaking_source() {
    let temp = tempdir().unwrap();
    let source = shaky_fixture(temp.path());
    let output = temp.path().join("stabilized.mp4");
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_shaky",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    let mut baseline = media_clip("itm_shaky_before", "med_shaky", 0, 2_000);
    baseline.visual = Some(full_visual());
    let mut stabilized = media_clip("itm_shaky_after", "med_shaky", 2_000, 2_000);
    stabilized.visual = Some(full_visual());
    stabilized.effects.push(video_effect(
        "fx_stabilize_e2e",
        "video.stabilize",
        BTreeMap::from([(
            "enabled".to_owned(),
            ParameterValue::Boolean { value: true },
        )]),
    ));
    canonical.project.sequences[0].tracks.push(track(
        "trk_stabilize",
        TrackKind::Video,
        0,
        vec![baseline, stabilized],
    ));
    let rendered = render(
        canonical,
        &BTreeMap::from([("med_shaky".to_owned(), source)]),
        &output,
    );
    assert!(rendered.command.filter_graph.unwrap().contains("deshake"));
    let changed = [0.5, 1.0, 1.5]
        .into_iter()
        .map(|second| {
            changed_channels(
                &rgb_frame(&output, second),
                &rgb_frame(&output, second + 2.0),
            )
        })
        .max()
        .unwrap();
    assert!(changed > 100, "stabilize changed channels={changed}");
}

fn effect_clip(id: &str, start: i64, effect: Option<EffectInstance>) -> Clip {
    let mut clip = media_clip(&format!("itm_{id}"), "med_effect_pattern", start, 1_000);
    clip.visual = Some(full_visual());
    clip.effects.extend(effect);
    clip
}

fn curve(id: &str, start: f64, end: f64) -> ParameterValue {
    ParameterValue::NumberCurve {
        value: Animatable::Keyframes {
            keyframes: vec![
                Keyframe {
                    id: KeyframeId::new(format!("kf_{id}_start")).unwrap(),
                    time: time(0),
                    value: start,
                    interpolation: Interpolation::EaseInOut,
                },
                Keyframe {
                    id: KeyframeId::new(format!("kf_{id}_end")).unwrap(),
                    time: time(1_000),
                    value: end,
                    interpolation: Interpolation::Linear,
                },
            ],
        },
    }
}

fn pattern_fixture(directory: &Path) -> PathBuf {
    let output = directory.join("effect-pattern.mp4");
    run(Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            &format!("testsrc2=s={WIDTH}x{HEIGHT}:r={FPS}:d=1"),
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(&output));
    output
}

fn shaky_fixture(directory: &Path) -> PathBuf {
    let output = directory.join("shaky.mp4");
    run(Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=s=112x70:r=10:d=2",
            "-vf",
            "crop=96:54:x='8+5*sin(n*1.7)':y='8+5*cos(n*1.3)'",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(&output));
    output
}
