use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{FrameObservation, MaskSpec, PixelFormat, RegionSpec};

use super::frame::same_geometry;
use super::{resolve_region, MetricError, PixelRect};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mask {
    pub rect: PixelRect,
    pub values: Vec<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub margin_left: u32,
    pub margin_top: u32,
    pub margin_right: u32,
    pub margin_bottom: u32,
}

pub fn mask(
    frame: &FrameObservation,
    reference: Option<&FrameObservation>,
    region: Option<&RegionSpec>,
    spec: &MaskSpec,
) -> Result<Mask, MetricError> {
    let rect = resolve_region(frame, region)?;
    if let Some(reference) = reference {
        same_geometry(frame, reference)?;
    }
    if matches!(spec, MaskSpec::Alpha { .. }) && frame.format != PixelFormat::Rgba8 {
        return Err(MetricError::MissingAlpha);
    }
    let mut values = Vec::with_capacity((rect.width * rect.height) as usize);
    for index in rect.indices(frame.width) {
        let pixel = frame.rgba(index);
        let selected = match spec {
            MaskSpec::Alpha { minimum } => pixel[3] >= *minimum,
            MaskSpec::Difference { minimum_delta, .. } => {
                let other = reference.ok_or(MetricError::EmptySelection)?.rgba(index);
                pixel
                    .iter()
                    .zip(other)
                    .take(3)
                    .any(|(left, right)| left.abs_diff(right) >= *minimum_delta)
            }
            MaskSpec::Luma { threshold, above } => {
                let value = luma(pixel);
                if *above {
                    value >= *threshold
                } else {
                    value <= *threshold
                }
            }
        };
        values.push(selected);
    }
    Ok(Mask { rect, values })
}

pub fn bounds(mask: &Mask, frame_width: u32, frame_height: u32) -> Option<Bounds> {
    let selected = mask
        .values
        .iter()
        .enumerate()
        .filter(|(_, value)| **value)
        .map(|(index, _)| {
            let local_x = index as u32 % mask.rect.width;
            let local_y = index as u32 / mask.rect.width;
            (mask.rect.x + local_x, mask.rect.y + local_y)
        });
    let mut left = frame_width;
    let mut top = frame_height;
    let mut right = 0_u32;
    let mut bottom = 0_u32;
    let mut found = false;
    for (x, y) in selected {
        left = left.min(x);
        top = top.min(y);
        right = right.max(x);
        bottom = bottom.max(y);
        found = true;
    }
    found.then(|| Bounds {
        x: left,
        y: top,
        width: right - left + 1,
        height: bottom - top + 1,
        margin_left: left,
        margin_top: top,
        margin_right: frame_width - right - 1,
        margin_bottom: frame_height - bottom - 1,
    })
}

fn luma(pixel: [u8; 4]) -> u8 {
    ((u32::from(pixel[0]) * 54 + u32::from(pixel[1]) * 183 + u32::from(pixel[2]) * 19) / 256) as u8
}
