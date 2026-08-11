use std::time::{Duration, Instant};

use crate::RuntimeError;

const MAX_EDGE: u32 = 16_384;
const MAX_FRAME_BYTES: u64 = 256 * 1024 * 1024;
const MAX_WALL_SECONDS: u64 = veac_artifact::MAX_MEDIA_DERIVATION_WALL_SECONDS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservationLimits {
    pub max_width: u32,
    pub max_height: u32,
    pub max_frame_bytes: u64,
    pub max_wall_seconds: u64,
}

impl ObservationLimits {
    pub fn validate(self) -> Result<Self, RuntimeError> {
        let valid = self.max_width > 0
            && self.max_width <= MAX_EDGE
            && self.max_height > 0
            && self.max_height <= MAX_EDGE
            && self.max_frame_bytes > 0
            && self.max_frame_bytes <= MAX_FRAME_BYTES
            && self.max_wall_seconds > 0
            && self.max_wall_seconds <= MAX_WALL_SECONDS;
        valid
            .then_some(self)
            .ok_or_else(|| RuntimeError::new("invalid media observation resource policy"))
    }

    pub(crate) fn deadline(self) -> Result<Instant, RuntimeError> {
        self.validate().and_then(|value| {
            Instant::now()
                .checked_add(Duration::from_secs(value.max_wall_seconds))
                .ok_or_else(|| RuntimeError::resource_limit("observation deadline overflows"))
        })
    }

    pub(crate) fn frame_bytes(
        self,
        width: u32,
        height: u32,
        bytes_per_pixel: u8,
    ) -> Result<u64, RuntimeError> {
        let bytes = u64::from(width)
            .checked_mul(u64::from(height))
            .and_then(|value| value.checked_mul(u64::from(bytes_per_pixel)))
            .filter(|value| *value <= self.max_frame_bytes);
        if width > self.max_width || height > self.max_height {
            return Err(RuntimeError::resource_limit(
                "observed frame exceeds its geometry limit",
            ));
        }
        bytes.ok_or_else(|| RuntimeError::resource_limit("observed frame exceeds its byte limit"))
    }
}

impl Default for ObservationLimits {
    fn default() -> Self {
        Self {
            max_width: 8_192,
            max_height: 8_192,
            max_frame_bytes: 128 * 1024 * 1024,
            max_wall_seconds: MAX_WALL_SECONDS,
        }
    }
}
