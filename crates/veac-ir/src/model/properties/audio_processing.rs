use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{BusId, RationalTime, TrackId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AudioProcessor {
    ParametricEq {
        bands: Vec<ParametricEqBand>,
    },
    HighPass {
        frequency_hz: f64,
        q: f64,
        poles: u8,
    },
    LowPass {
        frequency_hz: f64,
        q: f64,
        poles: u8,
    },
    Compressor(Compressor),
    Limiter(Limiter),
    Gate(Gate),
    Loudness(LoudnessTarget),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ParametricEqBand {
    pub frequency_hz: f64,
    pub gain_db: f64,
    pub q: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Compressor {
    pub threshold_db: f64,
    pub ratio: f64,
    pub attack_ms: f64,
    pub release_ms: f64,
    pub knee_db: f64,
    pub makeup_gain_db: f64,
    pub mix: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Limiter {
    pub ceiling_db: f64,
    pub attack_ms: f64,
    pub release_ms: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Gate {
    pub threshold_db: f64,
    pub ratio: f64,
    pub attack_ms: f64,
    pub release_ms: f64,
    pub range_db: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LoudnessTarget {
    pub integrated_lufs: f64,
    pub true_peak_dbtp: f64,
    pub loudness_range_lu: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AudioFadeCurve {
    Linear,
    EqualPower,
    Exponential,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AudioCrossfade {
    pub fade_in: RationalTime,
    pub fade_out: RationalTime,
    pub curve: AudioFadeCurve,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SidechainSource {
    Track { track_id: TrackId },
    Bus { bus_id: BusId },
}
