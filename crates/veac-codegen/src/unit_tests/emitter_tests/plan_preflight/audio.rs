use super::*;

#[test]
fn untrusted_audio_contracts_fail_in_preflight() {
    let mut processor = audio_plan();
    audio(&mut processor).processors = vec![AudioProcessor {
        id: AudioProcessorId::new("aud_invalid-hpf").unwrap(),
        kind: AudioProcessorKind::HighPass {
            frequency_hz: 48_000.0,
            q: 1.0,
            poles: 2,
        },
    }];
    assert_code(&processor, "PLAN_AUDIO_INVALID");

    let mut conflict = audio_plan();
    audio(&mut conflict).normalize = true;
    audio(&mut conflict).processors = vec![AudioProcessor {
        id: AudioProcessorId::new("aud_loudness-conflict").unwrap(),
        kind: AudioProcessorKind::Loudness(LoudnessTarget {
            integrated_lufs: -16.0,
            true_peak_dbtp: -1.5,
            loudness_range_lu: 11.0,
        }),
    }];
    assert_code(&conflict, "PLAN_AUDIO_INVALID");

    let mut fade = audio_plan();
    audio(&mut fade).crossfade = Some(AudioCrossfade {
        fade_in: time(400),
        fade_out: time(400),
        curve: AudioFadeCurve::EqualPower,
    });
    assert_code(&fade, "PLAN_AUDIO_INVALID");

    let mut sidechain = audio_plan();
    audio(&mut sidechain).sidechain = Some(ResolvedSidechain {
        relation_id: RelationId::new("rel_invalid_sidechain").unwrap(),
        source: SidechainSource::Track {
            track_id: TrackId::new("trk_missing").unwrap(),
        },
        threshold_db: -20.0,
        ratio: 4.0,
        attack_ms: 10.0,
        release_ms: 100.0,
        active_range: None,
    });
    assert_code(&sidechain, "PLAN_AUDIO_INVALID");
}
