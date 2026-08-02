use crate::authoring::{format_document, parse, ArtifactRecipe, VideoRateControl};

use super::project;

fn source(rate_control: &str) -> String {
    project(&format!(
        r#"  sequence main {{}}
  delivery preview {{
    sequence main;
    raster {{ canvas 1920px by 1080px; frame-rate 30fps; captions discard; }}
    artifact video preview {{
      target file "preview.mp4";
      mux mp4 {{ video h264 {{ {rate_control} }} audio none; }}
    }}
  }}"#
    ))
}

fn parsed(rate_control: &str) -> (VideoRateControl, String) {
    let document = parse(&source(rate_control)).unwrap();
    let ArtifactRecipe::Video(recipe) = &document.project.deliveries[0].artifacts[0].recipe else {
        panic!("video output expected")
    };
    (
        recipe.video.rate_control.clone(),
        format_document(&document),
    )
}

fn assert_diagnostic(rate_control: &str, code: &'static str, message: &str, span: &str) {
    let source = source(rate_control);
    let errors = parse(&source).unwrap_err();
    let diagnostic = errors
        .as_slice()
        .iter()
        .find(|value| value.code == code && value.message == message)
        .unwrap_or_else(|| panic!("missing {code}/{message}: {errors:?}"));
    assert_eq!(&source[diagnostic.span.start..diagnostic.span.end], span);
}

#[test]
fn explicit_crf_and_average_rates_round_trip() {
    let (crf, formatted) = parsed("rate-control crf { value 23; }");
    assert_eq!(crf, VideoRateControl::Crf { value: 23 });
    assert!(formatted.contains("rate-control crf"));
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);

    let (average, formatted) = parsed("rate-control average { target 8mbps; }");
    assert_eq!(
        average,
        VideoRateControl::Bitrate {
            target_bps: 8_000_000,
            max_bps: None,
            buffer_size_bits: None,
        }
    );
    assert!(formatted.contains("target 8mbps;"));
}

#[test]
fn capped_rate_preserves_rate_and_buffer_quantities() {
    let (rate, formatted) =
        parsed("rate-control capped { target 3mbps; max 3210kbps; buffer 6mbit; }");
    assert_eq!(
        rate,
        VideoRateControl::Bitrate {
            target_bps: 3_000_000,
            max_bps: Some(3_210_000),
            buffer_size_bits: Some(6_000_000),
        }
    );
    assert!(formatted.contains("buffer 6mbit;"));
}

#[test]
fn rate_modes_require_bodies_and_required_values() {
    for (rate, message) in [
        ("rate-control crf;", "crf requires a body"),
        (
            "rate-control average;",
            "average rate-control requires a body",
        ),
        (
            "rate-control capped;",
            "capped rate-control requires a body",
        ),
    ] {
        assert_diagnostic(rate, "AUTHORING_OUTPUT_RATE_CONTROL", message, rate);
    }
    for rate in [
        "rate-control crf {}",
        "rate-control average {}",
        "rate-control capped { target 1mbps; max 2mbps; }",
    ] {
        let errors = parse(&source(rate)).unwrap_err();
        assert!(errors
            .as_slice()
            .iter()
            .any(|value| value.code == "AUTHORING_REQUIRED_FIELD"));
    }
}

#[test]
fn rate_variant_shape_names_and_members_are_closed() {
    for (rate, code, message, span) in [
        (
            "rate-control 23;",
            "AUTHORING_OUTPUT_RATE_CONTROL",
            "rate-control requires a variant",
            "rate-control 23;",
        ),
        (
            "rate-control mystery;",
            "AUTHORING_OUTPUT_ENUM",
            "invalid rate-control variant 'mystery'",
            "mystery",
        ),
        (
            "rate-control crf { value 23; surprise 1; }",
            "AUTHORING_UNKNOWN_FIELD",
            "crf rate control does not support `surprise`",
            "surprise",
        ),
    ] {
        assert_diagnostic(rate, code, message, span);
    }
    assert_diagnostic(
        "rate-control lossless {}",
        "AUTHORING_OUTPUT_RATE_CONTROL",
        "lossless does not accept a body",
        "rate-control lossless {}",
    );
}
