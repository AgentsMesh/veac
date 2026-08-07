use crate::{
    Color, Point, Rect, TemporalColorChannel, TemporalEvaluationError, TemporalPointAxis,
    TemporalRectField, TemporalValue, TemporalVectorAxis, Vec2,
};

use super::support::{contract, error};

pub(in crate::temporal::evaluation) fn compose_vec2(
    x: &TemporalValue,
    y: &TemporalValue,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    match (x, y) {
        (TemporalValue::Scalar { value: x }, TemporalValue::Scalar { value: y }) => {
            Ok(TemporalValue::Vec2 {
                value: Vec2 { x: *x, y: *y },
            })
        }
        _ => Err(contract(pointer)),
    }
}

pub(in crate::temporal::evaluation) fn project_vec2(
    value: &TemporalValue,
    axis: TemporalVectorAxis,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    let TemporalValue::Vec2 { value } = value else {
        return Err(contract(pointer));
    };
    Ok(TemporalValue::Scalar {
        value: match axis {
            TemporalVectorAxis::X => value.x,
            TemporalVectorAxis::Y => value.y,
        },
    })
}

pub(in crate::temporal::evaluation) fn compose_point(
    x: &TemporalValue,
    y: &TemporalValue,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    match (x, y) {
        (TemporalValue::Length { value: x }, TemporalValue::Length { value: y }) => {
            Ok(TemporalValue::Point {
                value: Point { x: *x, y: *y },
            })
        }
        _ => Err(contract(pointer)),
    }
}

pub(in crate::temporal::evaluation) fn project_point(
    value: &TemporalValue,
    axis: TemporalPointAxis,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    let TemporalValue::Point { value } = value else {
        return Err(contract(pointer));
    };
    Ok(TemporalValue::Length {
        value: match axis {
            TemporalPointAxis::X => value.x,
            TemporalPointAxis::Y => value.y,
        },
    })
}

pub(in crate::temporal::evaluation) fn compose_rect(
    x: &TemporalValue,
    y: &TemporalValue,
    width: &TemporalValue,
    height: &TemporalValue,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    let (
        TemporalValue::Scalar { value: x },
        TemporalValue::Scalar { value: y },
        TemporalValue::Scalar { value: width },
        TemporalValue::Scalar { value: height },
    ) = (x, y, width, height)
    else {
        return Err(contract(pointer));
    };
    Ok(TemporalValue::Rect {
        value: Rect {
            x: *x,
            y: *y,
            width: *width,
            height: *height,
        },
    })
}

pub(in crate::temporal::evaluation) fn project_rect(
    value: &TemporalValue,
    field: TemporalRectField,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    let TemporalValue::Rect { value } = value else {
        return Err(contract(pointer));
    };
    Ok(TemporalValue::Scalar {
        value: match field {
            TemporalRectField::X => value.x,
            TemporalRectField::Y => value.y,
            TemporalRectField::Width => value.width,
            TemporalRectField::Height => value.height,
        },
    })
}

pub(in crate::temporal::evaluation) fn compose_color(
    red: &TemporalValue,
    green: &TemporalValue,
    blue: &TemporalValue,
    alpha: &TemporalValue,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    let (
        TemporalValue::Integer { value: red },
        TemporalValue::Integer { value: green },
        TemporalValue::Integer { value: blue },
        TemporalValue::Integer { value: alpha },
    ) = (red, green, blue, alpha)
    else {
        return Err(contract(pointer));
    };
    Ok(TemporalValue::Color {
        value: Color {
            red: channel(*red, pointer)?,
            green: channel(*green, pointer)?,
            blue: channel(*blue, pointer)?,
            alpha: channel(*alpha, pointer)?,
        },
    })
}

pub(in crate::temporal::evaluation) fn project_color(
    value: &TemporalValue,
    channel: TemporalColorChannel,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    let TemporalValue::Color { value } = value else {
        return Err(contract(pointer));
    };
    Ok(TemporalValue::Integer {
        value: i64::from(match channel {
            TemporalColorChannel::Red => value.red,
            TemporalColorChannel::Green => value.green,
            TemporalColorChannel::Blue => value.blue,
            TemporalColorChannel::Alpha => value.alpha,
        }),
    })
}

fn channel(value: i64, pointer: &str) -> Result<u8, TemporalEvaluationError> {
    u8::try_from(value).map_err(|_| error(pointer, "color channel is outside 0 through 255"))
}
