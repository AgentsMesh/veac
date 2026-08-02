use crate::authoring::{lower_document, parse, Diagnostics, Document, NumberLiteral};

use super::project;

const BODY: &str = r#"sequence main {
  layer visual picture {
    item blank {
      source generated transparent;
      record { at 0s; duration 1s; }
    }
  }
}"#;

type Mutation = fn(&mut Document);

fn document() -> Document {
    parse(&project(BODY)).unwrap()
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
fn u32_overflow_is_reported_for_canvas_sample_rate_and_frame_rate_denominator() {
    let cases: [(Mutation, &str); 3] = [
        (
            |document| document.project.settings.canvas.as_mut().unwrap().0 = raw("4294967296px"),
            "AUTHORING_LOWER_INTEGER_RANGE",
        ),
        (
            |document| document.project.settings.sample_rate = Some(raw("4294967296hz")),
            "AUTHORING_LOWER_INTEGER_RANGE",
        ),
        (
            |document| document.project.settings.frame_rate = Some(raw("1/4294967296fps")),
            "AUTHORING_LOWER_FRAME_RATE",
        ),
    ];
    for (mutate, code) in cases {
        let mut document = document();
        mutate(&mut document);
        let error = lower_document(&document).unwrap_err();
        assert_code(&error, code);
        assert!(error.as_slice().iter().any(|value| {
            value.message.contains("32-bit") || value.message.contains("rational")
        }));
    }
}

#[test]
fn decimal_frame_rate_is_lowered_as_an_exact_rational() {
    let mut decimal = document();
    decimal.project.settings.frame_rate = Some(raw("29.97fps"));
    let lowered = lower_document(&decimal).unwrap_or_else(|error| panic!("{error}"));
    let settings = &lowered.project.sequences[0].settings;
    assert_eq!(
        (
            settings.frame_rate.numerator,
            settings.frame_rate.denominator
        ),
        (2997, 100)
    );
}

#[test]
fn malformed_timebase_frame_rate_and_canvas_unit_keep_specific_diagnostics() {
    let mut bad_timebase = document();
    bad_timebase.project.settings.timebase = Some(raw("2/1000"));
    assert_code(
        &lower_document(&bad_timebase).unwrap_err(),
        "AUTHORING_LOWER_TIMEBASE",
    );

    let mut bad_frame_rate = document();
    bad_frame_rate.project.settings.frame_rate = Some(raw("30hz"));
    assert_code(
        &lower_document(&bad_frame_rate).unwrap_err(),
        "AUTHORING_LOWER_FRAME_RATE",
    );

    let mut bad_canvas = document();
    bad_canvas.project.settings.canvas.as_mut().unwrap().0 = raw("1920");
    let error = lower_document(&bad_canvas).unwrap_err();
    assert_code(&error, "AUTHORING_LOWER_UNIT");
    assert_code(&error, "AUTHORING_LOWER_REQUIRED");
}

#[test]
fn every_required_project_setting_reports_its_missing_field() {
    let cases: [fn(&mut Document); 4] = [
        |document| document.project.settings.timebase = None,
        |document| document.project.settings.canvas = None,
        |document| document.project.settings.frame_rate = None,
        |document| document.project.settings.sample_rate = None,
    ];
    for mutate in cases {
        let mut document = document();
        mutate(&mut document);
        assert_code(
            &lower_document(&document).unwrap_err(),
            "AUTHORING_LOWER_REQUIRED",
        );
    }
}
