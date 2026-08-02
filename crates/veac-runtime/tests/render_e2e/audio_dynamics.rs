use tempfile::tempdir;

use super::support::*;

#[test]
fn compressor_limiter_and_gate_change_measured_level_and_peak() {
    let temp = tempdir().unwrap();
    let source = tone_wav(temp.path(), "dynamic", 440, 1.0, 4.0);
    let baseline = temp.path().join("baseline.mp4");
    render_audio_chain(&source, 1_000, vec![], None, &baseline);
    let baseline_samples = audio_samples(&baseline, 0.2, 0.5);

    let compressed = temp.path().join("compressed.mp4");
    render_audio_chain(
        &source,
        1_000,
        vec![identified_processor(
            "compressor",
            AudioProcessorKind::Compressor(Compressor {
                threshold_db: -30.0,
                ratio: 20.0,
                attack_ms: 1.0,
                release_ms: 100.0,
                knee_db: 0.0,
                makeup_gain_db: 0.0,
                mix: 1.0,
            }),
        )],
        None,
        &compressed,
    );
    let compressed_samples = audio_samples(&compressed, 0.2, 0.5);
    assert!(rms_db(&compressed_samples) < rms_db(&baseline_samples) - 6.0);

    let limited = temp.path().join("limited.mp4");
    render_audio_chain(
        &source,
        1_000,
        vec![identified_processor(
            "limiter",
            AudioProcessorKind::Limiter(Limiter {
                ceiling_db: -12.0,
                attack_ms: 5.0,
                release_ms: 50.0,
            }),
        )],
        None,
        &limited,
    );
    assert!(peak(&audio_samples(&limited, 0.2, 0.5)) < 0.29);

    let quiet = tone_wav(temp.path(), "quiet", 440, 1.0, 0.2);
    let gated = temp.path().join("gated.mp4");
    render_audio_chain(
        &quiet,
        1_000,
        vec![identified_processor(
            "gate",
            AudioProcessorKind::Gate(Gate {
                threshold_db: -20.0,
                ratio: 20.0,
                attack_ms: 1.0,
                release_ms: 50.0,
                range_db: -80.0,
            }),
        )],
        None,
        &gated,
    );
    assert!(rms_db(&audio_samples(&gated, 0.2, 0.5)) < -55.0);
}

#[test]
fn loudness_target_is_repeatable_and_crossfade_shapes_clip_edges() {
    let temp = tempdir().unwrap();
    let source = tone_wav(temp.path(), "loudness", 440, 1.0, 0.4);
    let processor = identified_processor(
        "loudness",
        AudioProcessorKind::Loudness(LoudnessTarget {
            integrated_lufs: -16.0,
            true_peak_dbtp: -1.0,
            loudness_range_lu: 7.0,
        }),
    );
    let first = temp.path().join("loudness-a.mp4");
    let second = temp.path().join("loudness-b.mp4");
    render_audio_chain(&source, 1_000, vec![processor.clone()], None, &first);
    render_audio_chain(&source, 1_000, vec![processor], None, &second);
    let first_rms = rms_db(&audio_samples(&first, 0.2, 0.5));
    let second_rms = rms_db(&audio_samples(&second, 0.2, 0.5));
    assert!(
        (first_rms - second_rms).abs() < 0.05,
        "{first_rms} vs {second_rms}"
    );
    assert!(
        (-22.0..=-10.0).contains(&first_rms),
        "target RMS={first_rms}"
    );

    let faded = temp.path().join("faded.mp4");
    render_audio_chain(
        &source,
        1_000,
        vec![],
        Some(AudioCrossfade {
            fade_in: time(200),
            fade_out: time(200),
            curve: AudioFadeCurve::EqualPower,
        }),
        &faded,
    );
    let start = rms_db(&audio_samples(&faded, 0.01, 0.05));
    let middle = rms_db(&audio_samples(&faded, 0.45, 0.1));
    let end = rms_db(&audio_samples(&faded, 0.94, 0.05));
    assert!(start < middle - 8.0, "start={start}, middle={middle}");
    assert!(end < middle - 5.0, "end={end}, middle={middle}");
}

#[test]
fn normalize_effect_changes_only_its_authored_audio_range() {
    let temp = tempdir().unwrap();
    let source = tone_wav(temp.path(), "partial-normalize", 440, 2.0, 0.1);
    let baseline = temp.path().join("normalize-baseline.mp4");
    let normalized = temp.path().join("normalize-partial.mp4");
    render_normalize_effect(&source, false, &baseline);
    render_normalize_effect(&source, true, &normalized);

    let before = rms_db(&audio_samples(&normalized, 0.1, 0.2));
    let before_baseline = rms_db(&audio_samples(&baseline, 0.1, 0.2));
    let middle = rms_db(&audio_samples(&normalized, 0.7, 0.5));
    let middle_baseline = rms_db(&audio_samples(&baseline, 0.7, 0.5));
    let after = rms_db(&audio_samples(&normalized, 1.7, 0.2));
    let after_baseline = rms_db(&audio_samples(&baseline, 1.7, 0.2));
    assert!((before - before_baseline).abs() < 1.5, "before={before}");
    assert!(middle > middle_baseline + 5.0, "middle={middle}");
    assert!((after - after_baseline).abs() < 1.5, "after={after}");
}

fn render_normalize_effect(source: &Path, enabled: bool, output: &Path) {
    let mut canonical = project(true);
    canonical.project.materials.push(material(
        "med_partial_normalize",
        MaterialKind::Audio,
        StreamChoice::Disabled,
        StreamChoice::Auto,
    ));
    let mut clip = media_clip("itm_partial_normalize", "med_partial_normalize", 0, 2_000);
    clip.audio = Some(audio_properties(1.0));
    if enabled {
        clip.effects.push(EffectInstance {
            id: EffectId::new("fx_partial_normalize").unwrap(),
            effect_type: "audio.normalize".to_owned(),
            enabled: true,
            enable_range: Some(TimeRange::new(time(500), time(1_000)).unwrap()),
            parameters: BTreeMap::from([(
                "target_lufs".to_owned(),
                ParameterValue::Number { value: -8.0 },
            )]),
        });
    }
    canonical.project.sequences[0].tracks.push(track(
        "trk_partial_normalize",
        TrackKind::Audio,
        0,
        vec![clip],
    ));
    render(
        canonical,
        &BTreeMap::from([("med_partial_normalize".to_owned(), source.to_path_buf())]),
        output,
    );
}
