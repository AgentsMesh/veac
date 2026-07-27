use tempfile::tempdir;

use super::support::*;

#[test]
fn finite_repeat_replays_video_and_audio_instead_of_advancing_the_source() {
    let temp = tempdir().unwrap();
    let source = retime_fixture(temp.path());
    let mut canonical = project(true);
    canonical.project.materials.push(material(
        "med_repeat",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Auto,
    ));
    let mut clip = media_clip("itm_repeat", "med_repeat", 0, 2_000);
    clip.source_mapping = Some(linear_mapping(
        2,
        ratio(1, 1),
        FrameSynthesisPolicy::Nearest,
    ));
    clip.audio = Some(audio_properties(1.0));
    canonical.project.sequences[0].tracks.push(track(
        "trk_repeat",
        TrackKind::Video,
        0,
        vec![clip],
    ));
    let output = temp.path().join("repeat.mp4");
    let rendered = render(
        canonical,
        &BTreeMap::from([("med_repeat".to_owned(), source)]),
        &output,
    );

    assert_media_contract(&output, 1, 2.0);
    for second in [0.25, 0.75, 1.25, 1.75] {
        let pixel = rgb_at(&output, second, 48, 26);
        assert!(
            pixel[0] > 180 && pixel[1] < 80 && pixel[2] < 80,
            "{pixel:?}"
        );
    }
    assert!(rms_db(&audio_samples(&output, 1.2, 0.5)) > -35.0);
    let graph = rendered.command.filter_graph.unwrap();
    assert!(graph.contains("concat=n=2:v=1:a=0"));
    assert!(graph.contains("concat=n=2:v=0:a=1"));
}

#[test]
fn follow_speed_changes_pitch_with_the_authored_playback_rate() {
    let temp = tempdir().unwrap();
    let source = tone_fixture(temp.path(), "follow-speed", 440);
    let mut canonical = project(true);
    canonical.project.materials.push(material(
        "med_pitch",
        MaterialKind::Audio,
        StreamChoice::Disabled,
        StreamChoice::Auto,
    ));
    let mut clip = media_clip("itm_pitch", "med_pitch", 0, 2_000);
    clip.source_mapping = Some(linear_mapping(
        1,
        ratio(1, 2),
        FrameSynthesisPolicy::Nearest,
    ));
    let mut audio = audio_properties(1.0);
    audio.pitch_policy = PitchPolicy::FollowSpeed;
    clip.audio = Some(audio);
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_picture",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_picture", color(0, 0, 0), 0, 2_000)],
        ),
        track("trk_pitch", TrackKind::Audio, 1, vec![clip]),
    ]);
    let output = temp.path().join("follow-speed.mp4");
    let rendered = render(
        canonical,
        &BTreeMap::from([("med_pitch".to_owned(), source)]),
        &output,
    );

    assert_media_contract(&output, 1, 2.0);
    let samples = audio_samples(&output, 0.4, 0.8);
    let followed = tone_power(&samples, 220.0);
    let original = tone_power(&samples, 440.0);
    assert!(
        followed > 0.01 && followed > original * 8.0,
        "{followed} vs {original}"
    );
    assert!(rendered
        .command
        .filter_graph
        .unwrap()
        .contains("asetrate=48000*0.5,aresample=48000"));
}
