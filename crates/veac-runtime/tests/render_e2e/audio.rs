use tempfile::tempdir;
use veac_ir::StreamChoice;

use super::support::*;

#[test]
fn independent_audio_tracks_mix_after_record_time_silence() {
    let temp = tempdir().unwrap();
    let tone_440 = tone_fixture(temp.path(), "tone440", 440);
    let tone_880 = tone_fixture(temp.path(), "tone880", 880);
    let mut canonical = project(true);
    canonical.project.materials.extend([
        material(
            "med_tone440",
            MaterialKind::Audio,
            StreamChoice::Disabled,
            StreamChoice::Auto,
        ),
        material(
            "med_tone880",
            MaterialKind::Audio,
            StreamChoice::Disabled,
            StreamChoice::Auto,
        ),
    ]);
    let mut first = media_clip("itm_tone440", "med_tone440", 500, 1_000);
    first.audio = Some(audio_properties(0.6));
    let mut second = media_clip("itm_tone880", "med_tone880", 1_000, 1_000);
    second.audio = Some(audio_properties(0.6));
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_picture",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_picture", color(0, 0, 255), 0, 2_000)],
        ),
        track("trk_tone440", TrackKind::Audio, 1, vec![first]),
        track("trk_tone880", TrackKind::Audio, 2, vec![second]),
    ]);
    let assets = BTreeMap::from([
        ("med_tone440".to_owned(), tone_440),
        ("med_tone880".to_owned(), tone_880),
    ]);
    let output = temp.path().join("mix.mp4");
    let rendered = render(canonical, &assets, &output);

    assert_eq!(rendered.plan.inputs.len(), 2);
    assert!(rendered
        .plan
        .inputs
        .iter()
        .all(|input| input.video.is_none() && input.audio.is_some()));
    assert_media_contract(&output, 1, 2.0);
    let silence = audio_samples(&output, 0.15, 0.2);
    let only_440 = audio_samples(&output, 0.65, 0.2);
    let overlap = audio_samples(&output, 1.15, 0.2);
    let only_880 = audio_samples(&output, 1.65, 0.2);
    assert!(
        rms_db(&silence) < -55.0,
        "silence={} dB; graph={}",
        rms_db(&silence),
        rendered.command.filter_graph.as_deref().unwrap_or("<none>")
    );
    assert_tone_dominates(&only_440, 440.0, 880.0);
    assert_tone_dominates(&only_880, 880.0, 440.0);
    assert!(tone_power(&overlap, 440.0) > 0.01);
    assert!(tone_power(&overlap, 880.0) > 0.01);
}

fn assert_tone_dominates(samples: &[f64], expected: f64, absent: f64) {
    let expected_power = tone_power(samples, expected);
    let absent_power = tone_power(samples, absent);
    assert!(expected_power > 0.01, "tone {expected}: {expected_power}");
    assert!(
        expected_power > absent_power * 8.0,
        "{expected_power} vs {absent_power}"
    );
}
