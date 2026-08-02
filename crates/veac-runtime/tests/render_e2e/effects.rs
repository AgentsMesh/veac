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
