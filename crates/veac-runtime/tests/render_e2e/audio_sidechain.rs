use tempfile::tempdir;

use super::support::*;

#[test]
fn sidechain_ducking_reduces_only_target_frequency_during_source_activity() {
    let temp = tempdir().unwrap();
    let target_source = tone_wav(temp.path(), "target", 440, 2.0, 1.0);
    let control_source = tone_wav(temp.path(), "control", 2_500, 1.0, 4.0);
    let mut canonical = project(true);
    canonical.project.materials.extend([
        material(
            "med_target",
            MaterialKind::Audio,
            StreamChoice::Disabled,
            StreamChoice::Auto,
        ),
        material(
            "med_control",
            MaterialKind::Audio,
            StreamChoice::Disabled,
            StreamChoice::Auto,
        ),
    ]);
    let mut target = media_clip("itm_target", "med_target", 0, 2_000);
    target.audio = Some(audio_properties(1.0));
    let mut control = media_clip("itm_control", "med_control", 500, 1_000);
    control.audio = Some(audio_properties(1.0));
    canonical.project.sequences[0].tracks.extend([
        track("trk_target", TrackKind::Audio, 0, vec![target]),
        track("trk_control", TrackKind::Audio, 1, vec![control]),
    ]);
    add_sidechain(
        &mut canonical,
        "seq_main",
        RelationEndpoint::track(TrackId::new("trk_control").unwrap()),
        "itm_target",
        SidechainRelationParameters {
            threshold_db: -40.0,
            ratio: 20.0,
            attack_ms: 5.0,
            release_ms: 150.0,
            active_range: None,
        },
    );
    let assets = BTreeMap::from([
        ("med_target".to_owned(), target_source),
        ("med_control".to_owned(), control_source),
    ]);
    let output = temp.path().join("ducked.mp4");
    render(canonical, &assets, &output);

    let before = tone_power(&audio_samples(&output, 0.2, 0.2), 440.0);
    let during = tone_power(&audio_samples(&output, 0.8, 0.2), 440.0);
    let after = tone_power(&audio_samples(&output, 1.75, 0.2), 440.0);
    assert!(during < before * 0.55, "before={before}, during={during}");
    assert!(after > during * 1.6, "during={during}, after={after}");
}

#[test]
fn partial_sidechain_ducking_preserves_audio_outside_its_authored_range() {
    let temp = tempdir().unwrap();
    let target_source = tone_wav(temp.path(), "partial-target", 440, 2.0, 1.0);
    let control_source = tone_wav(temp.path(), "partial-control", 2_500, 2.0, 4.0);
    let mut canonical = project(true);
    canonical.project.materials.extend([
        material(
            "med_partial_target",
            MaterialKind::Audio,
            StreamChoice::Disabled,
            StreamChoice::Auto,
        ),
        material(
            "med_partial_control",
            MaterialKind::Audio,
            StreamChoice::Disabled,
            StreamChoice::Auto,
        ),
    ]);
    let mut target = media_clip("itm_partial_target", "med_partial_target", 0, 2_000);
    target.audio = Some(audio_properties(1.0));
    let mut control = media_clip("itm_partial_control", "med_partial_control", 0, 2_000);
    control.audio = Some(audio_properties(1.0));
    canonical.project.sequences[0].tracks.extend([
        track("trk_partial_target", TrackKind::Audio, 0, vec![target]),
        track("trk_partial_control", TrackKind::Audio, 1, vec![control]),
    ]);
    add_sidechain(
        &mut canonical,
        "seq_main",
        RelationEndpoint::track(TrackId::new("trk_partial_control").unwrap()),
        "itm_partial_target",
        SidechainRelationParameters {
            threshold_db: -40.0,
            ratio: 20.0,
            attack_ms: 5.0,
            release_ms: 50.0,
            active_range: Some(TimeRange::new(time(500), time(1_000)).unwrap()),
        },
    );
    let assets = BTreeMap::from([
        ("med_partial_target".to_owned(), target_source),
        ("med_partial_control".to_owned(), control_source),
    ]);
    let output = temp.path().join("partial-ducked.mp4");
    render(canonical, &assets, &output);

    let before = tone_power(&audio_samples(&output, 0.2, 0.2), 440.0);
    let during = tone_power(&audio_samples(&output, 0.8, 0.2), 440.0);
    let after = tone_power(&audio_samples(&output, 1.75, 0.2), 440.0);
    assert!(during < before * 0.55, "before={before}, during={during}");
    assert!(after > during * 1.6, "during={during}, after={after}");
    assert!(
        (after / before).clamp(0.0, 1.0) > 0.8,
        "before={before}, after={after}"
    );
}
