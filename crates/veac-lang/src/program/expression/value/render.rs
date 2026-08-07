use super::Value;
use crate::program::expression::{ExactNumber, UnitSuffix};

mod nominal;
mod structural;

impl Value {
    pub fn render(&self) -> String {
        match self {
            Self::Integer(value) => value.to_string(),
            Self::Scalar(value) => render_scalar(*value),
            Self::Time(value) => render_time(*value),
            Self::Length(value) => render_unit(*value, UnitSuffix::Pixels),
            Self::Percent(value) => render_unit(*value, UnitSuffix::Percent),
            Self::Angle(value) => render_unit(*value, UnitSuffix::Degrees),
            Self::Text(value) => crate::string_codec::quote(value),
            Self::Color(value) | Self::Identifier(value) => value.to_string(),
            Self::Bool(value) => value.to_string(),
            Self::Range(value) => value.render(),
            Self::Closure(value) => format!("<closure:{}>", value.value_type()),
            Self::List(value) => structural::list(value, structural::expression),
            Self::Map(value) => structural::map(value, structural::expression),
            Self::Tuple(value) => structural::tuple(value, structural::expression),
            Self::Struct(value) => nominal::structure(value, structural::expression),
            Self::Enum(value) => nominal::enumeration(value, structural::expression),
            Self::Domain(value) => format!("<{}>", value.domain_type()),
        }
    }
}

fn render_time(value: ExactNumber) -> String {
    for unit in UnitSuffix::TIME {
        let (numerator, denominator) = unit.scale();
        let factor = ExactNumber::new(i128::from(denominator), i128::from(numerator))
            .expect("unit scale must be non-zero");
        if let Some(scaled) = value.checked_mul(factor) {
            if scaled.denominator() == 1 {
                return format!("{}{}", scaled.numerator(), unit.as_str());
            }
        }
    }
    render_unit(value, UnitSuffix::Seconds)
}

fn render_scalar(value: ExactNumber) -> String {
    let mut rendered = value.render();
    if !rendered.contains('.') && !rendered.contains(" / ") {
        rendered.push_str(".0");
    }
    rendered
}

fn render_unit(value: ExactNumber, unit: UnitSuffix) -> String {
    let rendered = value.render();
    let suffix = unit.as_str();
    if let Some((numerator, denominator)) = rendered.split_once(" / ") {
        format!("{numerator}{suffix} / {denominator}.0")
    } else {
        format!("{rendered}{suffix}")
    }
}
