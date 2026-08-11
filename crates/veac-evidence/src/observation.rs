use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::RationalTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PixelFormat {
    Rgb8,
    Rgba8,
}

impl PixelFormat {
    pub fn channels(self) -> usize {
        match self {
            Self::Rgb8 => 3,
            Self::Rgba8 => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FrameObservation {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub actual_pts: RationalTime,
    pub data: Vec<u8>,
}

impl FrameObservation {
    pub fn validate(&self) -> Result<(), ObservationError> {
        let pixels = usize::try_from(self.width)
            .ok()
            .and_then(|width| {
                usize::try_from(self.height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .ok_or(ObservationError::InvalidDimensions)?;
        let expected = pixels
            .checked_mul(self.format.channels())
            .ok_or(ObservationError::InvalidDimensions)?;
        if self.width == 0 || self.height == 0 || !self.actual_pts.is_valid() {
            return Err(ObservationError::InvalidDimensions);
        }
        if self.data.len() != expected {
            return Err(ObservationError::InvalidPayload {
                expected,
                actual: self.data.len(),
            });
        }
        Ok(())
    }

    pub(crate) fn rgba(&self, pixel: usize) -> [u8; 4] {
        let start = pixel * self.format.channels();
        match self.format {
            PixelFormat::Rgb8 => [
                self.data[start],
                self.data[start + 1],
                self.data[start + 2],
                255,
            ],
            PixelFormat::Rgba8 => [
                self.data[start],
                self.data[start + 1],
                self.data[start + 2],
                self.data[start + 3],
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DecodeObservation {
    pub complete: bool,
    pub decoded_frames: u64,
    pub last_pts: Option<RationalTime>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LayerOrderObservation {
    pub upper_entity: String,
    pub lower_entity: String,
    pub upper_order: i64,
    pub lower_order: i64,
    pub coactive: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ObservationSet {
    pub frames: BTreeMap<String, FrameObservation>,
    pub decodes: BTreeMap<String, DecodeObservation>,
    pub layer_orders: BTreeMap<String, LayerOrderObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservationError {
    InvalidDimensions,
    InvalidPayload { expected: usize, actual: usize },
}

impl std::fmt::Display for ObservationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDimensions => write!(formatter, "frame dimensions or PTS are invalid"),
            Self::InvalidPayload { expected, actual } => {
                write!(
                    formatter,
                    "frame payload has {actual} bytes; expected {expected}"
                )
            }
        }
    }
}

impl std::error::Error for ObservationError {}
