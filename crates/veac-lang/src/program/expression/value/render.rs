use super::Value;
use crate::program::expression::ExactNumber;

impl Value {
    pub fn render(&self) -> String {
        match self {
            Self::Scalar(value) => value.render(),
            Self::Time(value) => render_time(*value),
            Self::Length(value) => render_unit(*value, "px"),
            Self::Percent(value) => render_unit(*value, "%"),
            Self::Angle(value) => render_unit(*value, "deg"),
            Self::Text(value) => crate::string_codec::quote(value),
            Self::Color(value) | Self::Identifier(value) => value.clone(),
            Self::Bool(value) => value.to_string(),
        }
    }

    pub(crate) fn render_source_literal(&self) -> Result<String, &'static str> {
        match self {
            Self::Scalar(value) => Ok(source_number(*value)),
            Self::Time(value) => source_time(*value),
            Self::Length(value) => Ok(format!("{}px", source_number(*value))),
            Self::Percent(value) => Ok(format!("{}%", source_number(*value))),
            Self::Angle(value) => Ok(format!("{}deg", source_number(*value))),
            Self::Text(value) => Ok(crate::string_codec::quote(value)),
            Self::Color(value) | Self::Identifier(value) => Ok(value.clone()),
            Self::Bool(value) => Ok(value.to_string()),
        }
    }
}

fn render_time(value: ExactNumber) -> String {
    for (factor, unit) in [(1_i128, "s"), (1_000, "ms"), (1_000_000, "us")] {
        if let Some(scaled) = value.checked_mul(ExactNumber::integer(factor)) {
            if scaled.denominator() == 1 {
                return format!("{}{}", scaled.numerator(), unit);
            }
        }
    }
    render_unit(value, "s")
}

fn render_unit(value: ExactNumber, unit: &str) -> String {
    let rendered = value.render();
    if let Some((numerator, denominator)) = rendered.split_once(" / ") {
        format!("{numerator}{unit} / {denominator}")
    } else {
        format!("{rendered}{unit}")
    }
}

fn source_time(value: ExactNumber) -> Result<String, &'static str> {
    let rendered = render_time(value);
    if rendered.contains(" / ") {
        Err("exact time cannot be represented by an s, ms, or us literal")
    } else {
        Ok(rendered)
    }
}

fn source_number(value: ExactNumber) -> String {
    let exact = value.render();
    if !exact.contains(" / ") {
        return exact;
    }
    let approximate = value.numerator() as f64 / value.denominator() as f64;
    let mut rendered = format!("{approximate:.64}");
    while rendered.ends_with('0') {
        rendered.pop();
    }
    if rendered.ends_with('.') {
        rendered.pop();
    }
    if value.denominator() != 1 && !rendered.contains('.') {
        rendered.push_str(".0");
    }
    rendered
}
