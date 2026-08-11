use crate::{FrameObservation, RegionSpace, RegionSpec};

use super::MetricError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl PixelRect {
    pub(crate) fn indices(self, frame_width: u32) -> impl Iterator<Item = usize> {
        (self.y..self.y + self.height).flat_map(move |y| {
            (self.x..self.x + self.width).map(move |x| (y * frame_width + x) as usize)
        })
    }
}

pub fn resolve_region(
    frame: &FrameObservation,
    region: Option<&RegionSpec>,
) -> Result<PixelRect, MetricError> {
    frame
        .validate()
        .map_err(|error| MetricError::InvalidFrame(error.to_string()))?;
    let rect = match region.map(|value| &value.space) {
        None => PixelRect {
            x: 0,
            y: 0,
            width: frame.width,
            height: frame.height,
        },
        Some(RegionSpace::Pixels {
            x,
            y,
            width,
            height,
        }) => PixelRect {
            x: *x,
            y: *y,
            width: *width,
            height: *height,
        },
        Some(RegionSpace::Normalized {
            x,
            y,
            width,
            height,
        }) => normalized(frame.width, frame.height, *x, *y, *width, *height),
    };
    let right = rect.x.checked_add(rect.width);
    let bottom = rect.y.checked_add(rect.height);
    if rect.width == 0
        || rect.height == 0
        || right.is_none_or(|value| value > frame.width)
        || bottom.is_none_or(|value| value > frame.height)
    {
        return Err(MetricError::RegionOutsideFrame);
    }
    Ok(rect)
}

fn normalized(width: u32, height: u32, x: f64, y: f64, w: f64, h: f64) -> PixelRect {
    let left = (x * f64::from(width)).floor() as u32;
    let top = (y * f64::from(height)).floor() as u32;
    let right = ((x + w) * f64::from(width)).ceil() as u32;
    let bottom = ((y + h) * f64::from(height)).ceil() as u32;
    PixelRect {
        x: left,
        y: top,
        width: right.saturating_sub(left),
        height: bottom.saturating_sub(top),
    }
}

pub(crate) fn same_geometry(
    left: &FrameObservation,
    right: &FrameObservation,
) -> Result<(), MetricError> {
    if (left.width, left.height) != (right.width, right.height) {
        return Err(MetricError::GeometryMismatch);
    }
    left.validate()
        .map_err(|error| MetricError::InvalidFrame(error.to_string()))?;
    right
        .validate()
        .map_err(|error| MetricError::InvalidFrame(error.to_string()))
}
