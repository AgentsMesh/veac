use crate::{
    AacEncoding, AudioProcessor, Compressor, Gate, Limiter, LoudnessTarget, Mp3Encoding,
    ParametricEqBand,
};

pub fn mp3_encoding_valid(value: &Mp3Encoding) -> bool {
    let kbps = value.bitrate_bps / 1_000;
    value.bitrate_bps % 1_000 == 0
        && match value.sample_rate_hz {
            32_000 | 44_100 | 48_000 => matches!(
                kbps,
                32 | 40 | 48 | 56 | 64 | 80 | 96 | 112 | 128 | 160 | 192 | 224 | 256 | 320
            ),
            16_000 | 22_050 | 24_000 => matches!(
                kbps,
                8 | 16 | 24 | 32 | 40 | 48 | 56 | 64 | 80 | 96 | 112 | 128 | 144 | 160
            ),
            8_000 | 11_025 | 12_000 => {
                matches!(kbps, 8 | 16 | 24 | 32 | 40 | 48 | 56 | 64)
            }
            _ => false,
        }
}

pub fn hls_aac_encoding_valid(value: &AacEncoding) -> bool {
    (8_000..=512_000).contains(&value.bitrate_bps)
        && matches!(value.sample_rate_hz, 32_000 | 44_100 | 48_000)
}

pub fn audio_processor_valid(value: &AudioProcessor, sample_rate: u32) -> bool {
    match value {
        AudioProcessor::ParametricEq { bands } => {
            !bands.is_empty()
                && bands.len() <= 32
                && bands.iter().all(|band| band_valid(*band, sample_rate))
        }
        AudioProcessor::HighPass {
            frequency_hz,
            q,
            poles,
        }
        | AudioProcessor::LowPass {
            frequency_hz,
            q,
            poles,
        } => {
            frequency(*frequency_hz, sample_rate)
                && finite_range(*q, 0.01, 100.0)
                && (1..=2).contains(poles)
        }
        AudioProcessor::Compressor(value) => compressor_valid(*value),
        AudioProcessor::Limiter(value) => limiter_valid(*value),
        AudioProcessor::Gate(value) => gate_valid(*value),
        AudioProcessor::Loudness(value) => loudness_valid(*value),
    }
}

fn band_valid(value: ParametricEqBand, sample_rate: u32) -> bool {
    frequency(value.frequency_hz, sample_rate)
        && db(value.gain_db, -24.0, 24.0)
        && finite_range(value.q, 0.01, 100.0)
}

fn compressor_valid(value: Compressor) -> bool {
    db(value.threshold_db, -60.0, 0.0)
        && finite_range(value.ratio, 1.0, 20.0)
        && finite_range(value.attack_ms, 0.01, 2000.0)
        && finite_range(value.release_ms, 0.01, 9000.0)
        && finite_range(value.knee_db, 0.0, 18.0)
        && db(value.makeup_gain_db, 0.0, 36.0)
        && finite_range(value.mix, 0.0, 1.0)
}

fn limiter_valid(value: Limiter) -> bool {
    db(value.ceiling_db, -24.0, 0.0)
        && finite_range(value.attack_ms, 0.1, 80.0)
        && finite_range(value.release_ms, 1.0, 8000.0)
}

fn gate_valid(value: Gate) -> bool {
    db(value.threshold_db, -120.0, 0.0)
        && finite_range(value.ratio, 1.0, 9000.0)
        && finite_range(value.attack_ms, 0.01, 9000.0)
        && finite_range(value.release_ms, 0.01, 9000.0)
        && db(value.range_db, -120.0, 0.0)
}

fn loudness_valid(value: LoudnessTarget) -> bool {
    db(value.integrated_lufs, -70.0, -5.0)
        && db(value.true_peak_dbtp, -9.0, 0.0)
        && finite_range(value.loudness_range_lu, 1.0, 50.0)
}

fn frequency(value: f64, sample_rate: u32) -> bool {
    value.is_finite() && value > 0.0 && value < f64::from(sample_rate) / 2.0
}

fn db(value: f64, minimum: f64, maximum: f64) -> bool {
    finite_range(value, minimum, maximum)
}

fn finite_range(value: f64, minimum: f64, maximum: f64) -> bool {
    value.is_finite() && (minimum..=maximum).contains(&value)
}
