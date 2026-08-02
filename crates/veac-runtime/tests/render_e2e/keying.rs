use tempfile::tempdir;

use super::support::*;

#[test]
fn luma_key_inversion_and_spill_suppression_change_real_frames() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("keying.mp4");
    let mut project = project(false);
    let background = solid_clip("itm_key_bg", color(0, 0, 255), 0, 4_000);
    let keyed = luma_clip("itm_luma_clear", 0, false);
    let inverted = luma_clip("itm_luma_invert", 1_000, true);
    let baseline = solid_clip("itm_spill_before", color(30, 210, 35), 2_000, 1_000);
    let mut suppressed = solid_clip("itm_spill_after", color(30, 210, 35), 3_000, 1_000);
    suppressed.effects.push(video_effect(
        "fx_spill_e2e",
        "video.chroma_spill",
        BTreeMap::from([
            (
                "color".to_owned(),
                ParameterValue::Color {
                    value: color(0, 255, 0),
                },
            ),
            ("amount".to_owned(), ParameterValue::Number { value: 1.0 }),
            ("range".to_owned(), ParameterValue::Number { value: 0.2 }),
        ]),
    ));
    project.project.sequences[0].tracks.extend([
        track("trk_key_bg", TrackKind::Video, 0, vec![background]),
        track(
            "trk_key_fg",
            TrackKind::Visual,
            1,
            vec![keyed, inverted, baseline, suppressed],
        ),
    ]);

    render(project, &BTreeMap::new(), &output);
    assert_media_contract(&output, 0, 4.0);
    assert_blue(rgb_at(&output, 0.5, WIDTH / 2, HEIGHT / 2));
    let opaque = rgb_at(&output, 1.5, WIDTH / 2, HEIGHT / 2);
    assert!(
        opaque.iter().all(|channel| *channel < 45),
        "pixel={opaque:?}"
    );
    let before = rgb_at(&output, 2.5, WIDTH / 2, HEIGHT / 2);
    let after = rgb_at(&output, 3.5, WIDTH / 2, HEIGHT / 2);
    assert!(
        after[1] + 35 < before[1],
        "before={before:?}, after={after:?}"
    );
}

#[test]
fn animated_chroma_similarity_changes_real_alpha_composition() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("chroma-curve.mp4");
    let mut project = project(false);
    let background = solid_clip("itm_chroma_bg", color(0, 0, 255), 0, 2_000);
    let mut foreground = solid_clip("itm_chroma_fg", color(30, 210, 35), 0, 2_000);
    foreground.visual = Some(full_visual());
    foreground.effects.push(video_effect(
        "fx_chroma_curve",
        "video.chroma_key",
        BTreeMap::from([
            (
                "color".to_owned(),
                ParameterValue::Color {
                    value: color(0, 255, 0),
                },
            ),
            (
                "similarity".to_owned(),
                ParameterValue::NumberCurve {
                    value: Animatable::Keyframes {
                        keyframes: vec![
                            Keyframe {
                                id: KeyframeId::new("kf_chroma_start").unwrap(),
                                time: time(0),
                                value: 0.00001,
                                interpolation: Interpolation::Linear,
                            },
                            Keyframe {
                                id: KeyframeId::new("kf_chroma_end").unwrap(),
                                time: time(2_000),
                                value: 0.6,
                                interpolation: Interpolation::Linear,
                            },
                        ],
                    },
                },
            ),
        ]),
    ));
    project.project.sequences[0].tracks.extend([
        track("trk_chroma_bg", TrackKind::Video, 0, vec![background]),
        track("trk_chroma_fg", TrackKind::Visual, 1, vec![foreground]),
    ]);
    render(project, &BTreeMap::new(), &output);
    let early = rgb_at(&output, 0.1, WIDTH / 2, HEIGHT / 2);
    let late = rgb_at(&output, 1.8, WIDTH / 2, HEIGHT / 2);
    assert!(early[1] > early[2] + 60, "early={early:?}");
    assert!(late[2] > late[1] + 80, "late={late:?}");
}

fn luma_clip(id: &str, start: i64, invert: bool) -> Clip {
    let mut clip = solid_clip(id, color(0, 0, 0), start, 1_000);
    clip.visual = Some(full_visual());
    clip.effects.push(video_effect(
        &format!("fx_{id}"),
        "video.luma_key",
        BTreeMap::from([
            (
                "threshold".to_owned(),
                ParameterValue::Number { value: 0.0 },
            ),
            (
                "tolerance".to_owned(),
                ParameterValue::Number { value: 0.12 },
            ),
            ("softness".to_owned(), ParameterValue::Number { value: 0.0 }),
            (
                "invert".to_owned(),
                ParameterValue::Boolean { value: invert },
            ),
        ]),
    ));
    clip
}

fn assert_blue(pixel: [u8; 3]) {
    assert!(pixel[2] > 170 && pixel[0] < 70, "pixel={pixel:?}");
}
