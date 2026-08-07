use veac_plan::canonical::*;

pub(super) fn families() -> Vec<(TemporalValue, TemporalValue)> {
    vec![
        (boolean(false), boolean(true)),
        (integer(1), integer(2)),
        (scalar(1.0), scalar(2.0)),
        (time(100), time(200)),
        (
            length(1.0, LengthUnit::Pixels),
            length(2.0, LengthUnit::Pixels),
        ),
        (angle(10.0), angle(20.0)),
        (vector(1.0), vector(2.0)),
        (point(1.0), point(2.0)),
        (rect(1.0), rect(2.0)),
        (color(1), color(2)),
        (text("same"), text("same")),
    ]
}

pub(super) fn length(value: f64, unit: LengthUnit) -> TemporalValue {
    TemporalValue::Length {
        value: Length { value, unit },
    }
}

fn boolean(value: bool) -> TemporalValue {
    TemporalValue::Boolean { value }
}

fn integer(value: i64) -> TemporalValue {
    TemporalValue::Integer { value }
}

fn scalar(value: f64) -> TemporalValue {
    TemporalValue::Scalar { value }
}

fn time(value: i64) -> TemporalValue {
    TemporalValue::Time {
        value: RationalTime::new(value, 600).unwrap(),
    }
}

fn angle(degrees: f64) -> TemporalValue {
    TemporalValue::Angle { degrees }
}

fn vector(value: f64) -> TemporalValue {
    TemporalValue::Vec2 {
        value: Vec2 { x: value, y: value },
    }
}

fn point(value: f64) -> TemporalValue {
    TemporalValue::Point {
        value: Point {
            x: Length {
                value,
                unit: LengthUnit::Pixels,
            },
            y: Length {
                value,
                unit: LengthUnit::Pixels,
            },
        },
    }
}

fn rect(value: f64) -> TemporalValue {
    TemporalValue::Rect {
        value: Rect {
            x: value,
            y: value,
            width: value,
            height: value,
        },
    }
}

fn color(value: u8) -> TemporalValue {
    TemporalValue::Color {
        value: Color {
            red: value,
            green: value,
            blue: value,
            alpha: value,
        },
    }
}

fn text(value: &str) -> TemporalValue {
    TemporalValue::Text {
        value: value.to_owned(),
    }
}
