mod curve;
mod interpolation;
mod values;

pub(super) use curve::{angle, length, percent, point, rect, scalar, vector};
pub(super) use values::{
    angle_value, length_value, percent_value, point_value, rect_value, scalar_value, vector_value,
};

use super::error::ExecutableLowerError;

fn invalid() -> ExecutableLowerError {
    ExecutableLowerError::lower(
        "EXECUTABLE_LOWER_ANIMATION",
        "a typed animation is invalid or cannot be represented in canonical IR",
    )
}
