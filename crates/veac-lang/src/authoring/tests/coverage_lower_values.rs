use crate::authoring::{lower_document, parse, Diagnostics, Document, NumberLiteral};

use super::project;

fn document() -> Document {
    parse(&project(
        r#"sequence main {
  layer visual picture {
    item blank {
      source generated transparent;
      record { at 0s; duration 1s; }
    }
  }
}"#,
    ))
    .unwrap()
}

fn raw(value: &str) -> NumberLiteral {
    NumberLiteral {
        raw: value.to_owned(),
        span: Default::default(),
    }
}

fn assert_code(error: &Diagnostics, code: &str) {
    assert!(
        error.as_slice().iter().any(|value| value.code == code),
        "missing {code}: {error}"
    );
}

#[test]
fn microseconds_convert_exactly_to_the_project_timebase() {
    let mut document = document();
    document.project.sequences[0].layers[0].items[0].record.at = raw("1000us");
    let envelope = lower_document(&document).unwrap_or_else(|error| panic!("{error}"));
    let start = envelope.project.sequences[0].tracks[0].clips[0]
        .record_range
        .start;
    assert_eq!((start.value, start.timescale), (1000, 1_000_000));
}

#[test]
fn invalid_time_units_numbers_and_sub_tick_values_have_distinct_diagnostics() {
    for (value, code) in [
        ("1min", "AUTHORING_LOWER_TIME_UNIT"),
        ("abc", "AUTHORING_LOWER_NUMBER"),
        ("1.5ms", "AUTHORING_LOWER_TIME_PRECISION"),
    ] {
        let mut document = document();
        if value == "1.5ms" {
            document.project.settings.timebase = Some(raw("1/1000"));
        }
        document.project.sequences[0].layers[0].items[0].record.at = raw(value);
        assert_code(&lower_document(&document).unwrap_err(), code);
    }
}

#[test]
fn layer_order_requires_an_i32_integer() {
    for (value, code) in [
        ("1.5", "AUTHORING_LOWER_INTEGER"),
        ("2147483648", "AUTHORING_LOWER_INTEGER_RANGE"),
    ] {
        let mut document = document();
        document.project.sequences[0].layers[0].order = Some(raw(value));
        assert_code(&lower_document(&document).unwrap_err(), code);
    }

    let mut negative = document();
    negative.project.sequences[0].layers[0].order = Some(raw("-7"));
    let envelope = lower_document(&negative).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(envelope.project.sequences[0].tracks[0].order, -7);
}
