use crate::{ColorMatrix, ColorRange, ColorSpace};

/// Structural color-space combinations supported by the canonical conversion contract.
pub fn color_space_valid(value: ColorSpace) -> bool {
    value.matrix != ColorMatrix::Rgb || value.range == ColorRange::Full
}
