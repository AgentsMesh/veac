use std::collections::BTreeMap;

use crate::{BundleError, FrameObservation, PixelFormat};

const MAX_SHEET_BYTES: usize = 256 * 1024 * 1024;

pub(crate) fn frame(value: &FrameObservation) -> Result<Vec<u8>, BundleError> {
    value
        .validate()
        .map_err(|error| BundleError::InvalidContract(error.to_string()))?;
    encode(value.width, value.height, value.format, &value.data)
}

pub(crate) fn contact_sheet(
    frames: &BTreeMap<String, FrameObservation>,
) -> Result<Option<Vec<u8>>, BundleError> {
    if frames.is_empty() {
        return Ok(None);
    }
    for frame in frames.values() {
        frame
            .validate()
            .map_err(|error| BundleError::InvalidContract(error.to_string()))?;
    }
    let cell_width = frames.values().map(|value| value.width).max().unwrap_or(1);
    let cell_height = frames.values().map(|value| value.height).max().unwrap_or(1);
    let columns = u32::try_from(frames.len().min(4)).unwrap_or(1);
    let rows = u32::try_from(frames.len())
        .unwrap_or(u32::MAX)
        .div_ceil(columns);
    let width = sheet_dimension(cell_width, columns, "width")?;
    let height = sheet_dimension(cell_height, rows, "height")?;
    let length = sheet_length(width, height)?;
    let mut data = vec![0; length];
    for (index, frame) in frames.values().enumerate() {
        copy_frame(&mut data, width, cell_width, cell_height, index, frame);
    }
    encode(width, height, PixelFormat::Rgba8, &data).map(Some)
}

fn sheet_dimension(value: u32, count: u32, label: &str) -> Result<u32, BundleError> {
    value
        .checked_mul(count)
        .ok_or_else(|| BundleError::InvalidContract(format!("contact sheet {label} overflowed")))
}

fn sheet_length(width: u32, height: u32) -> Result<usize, BundleError> {
    usize::try_from(width)
        .ok()
        .and_then(|value| value.checked_mul(height as usize))
        .and_then(|value| value.checked_mul(4))
        .filter(|value| *value <= MAX_SHEET_BYTES)
        .ok_or_else(|| BundleError::InvalidContract("contact sheet exceeds its budget".into()))
}

fn copy_frame(
    target: &mut [u8],
    target_width: u32,
    cell_width: u32,
    cell_height: u32,
    index: usize,
    frame: &FrameObservation,
) {
    let column = index as u32 % 4;
    let row = index as u32 / 4;
    for y in 0..frame.height.min(cell_height) {
        for x in 0..frame.width.min(cell_width) {
            let source = (y * frame.width + x) as usize;
            let destination =
                (((row * cell_height + y) * target_width + column * cell_width + x) * 4) as usize;
            target[destination..destination + 4].copy_from_slice(&frame.rgba(source));
        }
    }
}

fn encode(
    width: u32,
    height: u32,
    format: PixelFormat,
    data: &[u8],
) -> Result<Vec<u8>, BundleError> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, width, height);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_color(match format {
        PixelFormat::Rgb8 => png::ColorType::Rgb,
        PixelFormat::Rgba8 => png::ColorType::Rgba,
    });
    let mut writer = match encoder.write_header() {
        Ok(writer) => writer,
        Err(error) => return Err(BundleError::Encode(error.to_string())),
    };
    if let Err(error) = writer.write_image_data(data) {
        return Err(BundleError::Encode(error.to_string()));
    }
    drop(writer);
    Ok(bytes)
}

#[cfg(test)]
#[path = "png/coverage_tests.rs"]
mod coverage_tests;
