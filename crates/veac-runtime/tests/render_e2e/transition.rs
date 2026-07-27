use tempfile::tempdir;
use veac_ir::StreamChoice;

use super::support::*;

#[test]
fn centered_cross_dissolve_preserves_video_audio_and_duration() {
    let temp = tempdir().unwrap();
    let tone_440 = tone_fixture(temp.path(), "transition440", 440);
    let tone_880 = tone_fixture(temp.path(), "transition880", 880);
    let mut canonical = project(true);
    canonical.project.materials.extend([
        material(
            "med_transition440",
            MaterialKind::Audio,
            StreamChoice::Disabled,
            StreamChoice::Auto,
        ),
        material(
            "med_transition880",
            MaterialKind::Audio,
            StreamChoice::Disabled,
            StreamChoice::Auto,
        ),
    ]);
    let red = solid_clip("itm_red", color(255, 0, 0), 0, 1_000);
    let blue = solid_clip("itm_blue", color(0, 0, 255), 1_000, 1_000);
    let mut first_tone = media_clip("itm_audio440", "med_transition440", 0, 1_000);
    first_tone.audio = Some(audio_properties(0.7));
    let mut second_tone = media_clip("itm_audio880", "med_transition880", 1_000, 1_000);
    second_tone.audio = Some(audio_properties(0.7));
    canonical.project.sequences[0].tracks.extend([
        track("trk_transition_video", TrackKind::Video, 0, vec![red, blue]),
        track(
            "trk_transition_audio",
            TrackKind::Audio,
            1,
            vec![first_tone, second_tone],
        ),
    ]);
    add_transition(
        &mut canonical,
        "seq_main",
        "itm_red",
        "itm_blue",
        cross_dissolve(),
    );
    add_transition(
        &mut canonical,
        "seq_main",
        "itm_audio440",
        "itm_audio880",
        cross_dissolve(),
    );
    let assets = BTreeMap::from([
        ("med_transition440".to_owned(), tone_440),
        ("med_transition880".to_owned(), tone_880),
    ]);
    let output = temp.path().join("transition.mp4");
    let rendered = render(canonical, &assets, &output);

    let transitions: Vec<_> = rendered
        .plan
        .sequences
        .last()
        .unwrap()
        .tracks
        .iter()
        .flat_map(|track| &track.transitions)
        .collect();
    assert_eq!(transitions.len(), 2);
    assert!(transitions.iter().all(|transition| {
        transition.record_window.start == time(750)
            && transition.record_window.duration == time(500)
            && transition.outgoing_handle.duration == time(250)
            && transition.incoming_handle.duration == time(250)
    }));
    assert_media_contract(&output, 1, 2.0);
    assert_red(rgb_at(&output, 0.5, WIDTH / 2, HEIGHT / 2));
    let middle = rgb_at(&output, 1.0, WIDTH / 2, HEIGHT / 2);
    assert!(middle[0] > 70 && middle[2] > 70, "middle={middle:?}");
    assert_blue(rgb_at(&output, 1.5, WIDTH / 2, HEIGHT / 2));

    let before = audio_samples(&output, 0.5, 0.2);
    let boundary = audio_samples(&output, 0.94, 0.12);
    let after = audio_samples(&output, 1.4, 0.2);
    assert_tone(&before, 440.0, 880.0);
    assert_tone(&after, 880.0, 440.0);
    assert!(rms_db(&boundary) < rms_db(&before) - 4.0);
    let peak = boundary.iter().copied().map(f64::abs).fold(0.0, f64::max);
    let max_step = boundary
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).abs())
        .fold(0.0, f64::max);
    assert!(
        peak < 0.2 && max_step < 0.09,
        "peak={peak}, step={max_step}"
    );
}

#[test]
fn edge_aligned_transitions_sample_real_endpoint_frames() {
    let temp = tempdir().unwrap();
    for (alignment, middle_time, name) in [
        (TransitionAlignment::BeforeCut, 0.75, "before"),
        (TransitionAlignment::AfterCut, 1.25, "after"),
    ] {
        let mut canonical = project(false);
        let red = solid_clip("itm_edge_red", color(255, 0, 0), 0, 1_000);
        let transition = Transition {
            kind: TransitionKind::Dissolve,
            duration: time(500),
            alignment,
        };
        let blue = solid_clip("itm_edge_blue", color(0, 0, 255), 1_000, 1_000);
        canonical.project.sequences[0].tracks.push(track(
            "trk_edge_transition",
            TrackKind::Video,
            0,
            vec![red, blue],
        ));
        add_transition(
            &mut canonical,
            "seq_main",
            "itm_edge_red",
            "itm_edge_blue",
            transition,
        );
        let output = temp.path().join(format!("transition-{name}.mp4"));
        render(canonical, &BTreeMap::new(), &output);
        assert_red(rgb_at(&output, 0.25, WIDTH / 2, HEIGHT / 2));
        let middle = rgb_at(&output, middle_time, WIDTH / 2, HEIGHT / 2);
        assert!(middle[0] > 55 && middle[2] > 55, "{name}={middle:?}");
        assert_blue(rgb_at(&output, 1.75, WIDTH / 2, HEIGHT / 2));
    }
}

fn cross_dissolve() -> Transition {
    Transition {
        kind: TransitionKind::Dissolve,
        duration: time(500),
        alignment: TransitionAlignment::Centered,
    }
}

fn assert_tone(samples: &[f64], expected: f64, absent: f64) {
    let expected = tone_power(samples, expected);
    let absent = tone_power(samples, absent);
    assert!(expected > 0.01 && expected > absent * 8.0);
}

fn assert_red(pixel: [u8; 3]) {
    assert!(pixel[0] > 180 && pixel[2] < 40, "pixel={pixel:?}");
}

fn assert_blue(pixel: [u8; 3]) {
    assert!(pixel[2] > 180 && pixel[0] < 40, "pixel={pixel:?}");
}
