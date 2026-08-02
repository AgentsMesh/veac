use veac_plan::canonical::*;

use super::super::support::time;

pub(super) fn complete_audio() -> AudioProperties {
    let mut audio = base_audio();
    audio.processors = vec![
        processor(
            "aud_tone-shaper",
            AudioProcessorKind::ParametricEq {
                bands: vec![ParametricEqBand {
                    id: EqBandId::new("eqb_presence").unwrap(),
                    frequency_hz: 1_000.0,
                    gain_db: 3.0,
                    q: 1.2,
                }],
            },
        ),
        processor(
            "aud_cleanup-hpf",
            AudioProcessorKind::HighPass {
                frequency_hz: 80.0,
                q: 0.707,
                poles: 2,
            },
        ),
        processor(
            "aud_cleanup-lpf",
            AudioProcessorKind::LowPass {
                frequency_hz: 18_000.0,
                q: 0.707,
                poles: 2,
            },
        ),
        processor(
            "aud_compressor",
            AudioProcessorKind::Compressor(Compressor {
                threshold_db: -18.0,
                ratio: 4.0,
                attack_ms: 10.0,
                release_ms: 200.0,
                knee_db: 6.0,
                makeup_gain_db: 3.0,
                mix: 1.0,
            }),
        ),
        processor(
            "aud_limiter",
            AudioProcessorKind::Limiter(Limiter {
                ceiling_db: -1.0,
                attack_ms: 5.0,
                release_ms: 50.0,
            }),
        ),
        processor(
            "aud_gate",
            AudioProcessorKind::Gate(Gate {
                threshold_db: -45.0,
                ratio: 2.0,
                attack_ms: 5.0,
                release_ms: 150.0,
                range_db: -80.0,
            }),
        ),
        processor(
            "aud_loudness",
            AudioProcessorKind::Loudness(LoudnessTarget {
                integrated_lufs: -16.0,
                true_peak_dbtp: -1.0,
                loudness_range_lu: 7.0,
            }),
        ),
    ];
    audio.crossfade = Some(AudioCrossfade {
        fade_in: time(30),
        fade_out: time(30),
        curve: AudioFadeCurve::EqualPower,
    });
    audio
}

pub(super) fn base_audio() -> AudioProperties {
    AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: vec![],
        crossfade: None,
    }
}

pub(super) fn enable_audio(project: &mut ProjectEnvelope) {
    project.project.render_configs[0]
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
}

fn processor(id: &str, kind: AudioProcessorKind) -> AudioProcessor {
    AudioProcessor {
        id: AudioProcessorId::new(id).unwrap(),
        kind,
    }
}
