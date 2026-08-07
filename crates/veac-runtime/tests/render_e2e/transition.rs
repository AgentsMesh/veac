use tempfile::tempdir;
use veac_ir::StreamChoice;

use super::support::*;

#[test]
fn centered_cross_dissolve_uses_full_video_and_audio_overlap() {
    let temp = tempdir().unwrap();
    let red_source = color_tone_fixture(temp.path(), "transition-red", "red", 440);
    let blue_source = color_tone_fixture(temp.path(), "transition-blue", "blue", 880);
    let mut canonical = project(true);
    canonical.project.materials.extend([
        material(
            "med_red",
            MaterialKind::Video,
            StreamChoice::Auto,
            StreamChoice::Auto,
        ),
        material(
            "med_blue",
            MaterialKind::Video,
            StreamChoice::Auto,
            StreamChoice::Auto,
        ),
    ]);
    let red = av_clip("itm_red", "med_red", 0);
    let blue = av_clip("itm_blue", "med_blue", 750);
    canonical.project.sequences[0].tracks.push(track(
        "trk_transition_video",
        TrackKind::Video,
        0,
        vec![red, blue],
    ));
    add_transition(
        &mut canonical,
        "seq_main",
        "itm_red",
        "itm_blue",
        cross_dissolve(),
    );
    let assets = BTreeMap::from([
        ("med_red".to_owned(), red_source),
        ("med_blue".to_owned(), blue_source),
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
    assert_eq!(transitions.len(), 1);
    assert!(transitions.iter().all(|transition| {
        transition.record_window.start == time(750)
            && transition.record_window.duration == time(500)
            && transition.outgoing_range == range(750, 500)
            && transition.incoming_range == range(0, 500)
    }));
    assert_media_contract(&output, 1, 2.0);
    assert_red(rgb_at(&output, 0.5, WIDTH / 2, HEIGHT / 2));
    let middle = rgb_at(&output, 1.0, WIDTH / 2, HEIGHT / 2);
    assert!(middle[0] > 70 && middle[2] > 70, "middle={middle:?}");
    assert_blue(rgb_at(&output, 1.5, WIDTH / 2, HEIGHT / 2));

    let before = audio_samples(&output, 0.5, 0.15);
    let boundary = audio_samples(&output, 0.94, 0.12);
    let after = audio_samples(&output, 1.45, 0.15);
    assert_tone(&before, 440.0, 880.0);
    assert_tone(&after, 880.0, 440.0);
    let low = tone_power(&boundary, 440.0);
    let high = tone_power(&boundary, 880.0);
    assert!(low > 0.002 && high > 0.002, "low={low}, high={high}");
    assert!(
        (0.4..=2.5).contains(&(low / high)),
        "low={low}, high={high}"
    );
}

fn av_clip(id: &str, material: &str, start: i64) -> Clip {
    let mut clip = media_clip(id, material, start, 1_250);
    clip.visual = Some(full_visual());
    clip.audio = Some(audio_properties(0.7));
    clip
}

fn cross_dissolve() -> Transition {
    Transition {
        kind: TransitionKind::Dissolve,
        duration: time(500),
        alignment: TransitionAlignment::Centered,
    }
}

fn range(start: i64, duration: i64) -> TimeRange {
    TimeRange::new(time(start), time(duration)).unwrap()
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
