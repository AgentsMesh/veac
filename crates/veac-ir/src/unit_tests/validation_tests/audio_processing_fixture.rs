use crate::*;

pub(super) fn complete_chain() -> Vec<AudioProcessor> {
    vec![
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
    ]
}

fn processor(id: &str, kind: AudioProcessorKind) -> AudioProcessor {
    AudioProcessor {
        id: AudioProcessorId::new(id).unwrap(),
        kind,
    }
}
