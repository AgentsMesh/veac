use tempfile::tempdir;

use super::support::*;

#[test]
fn transparent_video_preserves_background_and_silence_emits_quiet_audio() {
    let temp = tempdir().unwrap();
    let mut transparent = solid_clip("itm_transparent", color(0, 0, 0), 0, 1_000);
    transparent.source = ClipSource::Generated {
        generator: Generator::Transparent,
    };
    let mut silence = solid_clip("itm_silence", color(0, 0, 0), 0, 1_000);
    silence.source = ClipSource::Generated {
        generator: Generator::Silence,
    };
    silence.audio = Some(audio_properties(1.0));
    let mut canonical = project(true);
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_background",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_background", color(15, 120, 230), 0, 1_000)],
        ),
        track("trk_transparent", TrackKind::Visual, 1, vec![transparent]),
        track("trk_silence", TrackKind::Audio, 2, vec![silence]),
    ]);
    let output = temp.path().join("transparent-silence.mp4");
    render(canonical, &BTreeMap::new(), &output);

    assert_media_contract(&output, 1, 1.0);
    let visible_background = rgb_at(&output, 0.5, 48, 27);
    assert!(
        visible_background[0] < 55
            && (75..=155).contains(&visible_background[1])
            && visible_background[2] > 180,
        "background={visible_background:?}"
    );
    let samples = audio_samples(&output, 0.2, 0.5);
    assert!(!samples.is_empty(), "silence must produce an audio stream");
    assert!(rms_db(&samples) < -80.0, "silence={} dB", rms_db(&samples));
}
