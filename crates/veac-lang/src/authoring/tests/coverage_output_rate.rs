use crate::authoring::{format_document, parse, OutputEncoding, VideoRateControl};

use super::project;

fn source(rate_control: &str) -> String {
    project(&format!(
        r#"  sequence main {{}}
  output video preview {{
    sequence main;
    file-name "preview.mp4";
    encoding {{ video {{ {rate_control} }} audio none; }}
  }}"#
    ))
}

fn parsed(rate_control: &str) -> (VideoRateControl, String) {
    let document = parse(&source(rate_control)).unwrap();
    let OutputEncoding::Video(encoding) = &document.project.outputs[0].encoding else {
        panic!("video output expected")
    };
    (
        encoding.video.rate_control.clone(),
        format_document(&document),
    )
}

fn assert_diagnostic(rate_control: &str, code: &'static str, message: &str, spanned_text: &str) {
    let source = source(rate_control);
    let errors = parse(&source).unwrap_err();
    let [diagnostic] = errors.as_slice() else {
        panic!("expected one diagnostic, got {errors:?}")
    };
    assert_eq!(
        (diagnostic.code, diagnostic.message.as_str()),
        (code, message)
    );
    assert_eq!(
        &source[diagnostic.span.start..diagnostic.span.end],
        spanned_text
    );
}

#[test]
fn empty_crf_uses_its_default_and_formats_the_value_explicitly() {
    let (rate_control, formatted) = parsed("rate-control crf {}");
    assert_eq!(rate_control, VideoRateControl::Crf { value: 23 });
    assert_eq!(
        formatted
            .matches("        rate-control crf {\n          value 23;\n        }")
            .count(),
        1
    );
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
}

#[test]
fn empty_bitrate_uses_its_defaults_and_formats_only_the_target() {
    let (rate_control, formatted) = parsed("rate-control bitrate {}");
    assert_eq!(
        rate_control,
        VideoRateControl::Bitrate {
            target_bps: 8_000_000,
            max_bps: None,
            buffer_bps: None,
        }
    );
    assert_eq!(
        formatted
            .matches("        rate-control bitrate {\n          target-bps 8000000;\n        }")
            .count(),
        1
    );
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
}

#[test]
fn crf_and_bitrate_require_bodies() {
    for (rate_control, message) in [
        ("rate-control crf;", "crf requires a body"),
        ("rate-control bitrate;", "bitrate requires a body"),
    ] {
        assert_diagnostic(
            rate_control,
            "AUTHORING_OUTPUT_RATE_CONTROL",
            message,
            rate_control,
        );
    }
}

#[test]
fn lossless_rejects_a_body() {
    assert_diagnostic(
        "rate-control lossless {}",
        "AUTHORING_OUTPUT_RATE_CONTROL",
        "lossless does not accept a body",
        "rate-control lossless {}",
    );
}

#[test]
fn rate_control_variant_shape_and_name_diagnostics_are_exact() {
    for (rate_control, code, message, spanned_text) in [
        (
            "rate-control 23;",
            "AUTHORING_OUTPUT_RATE_CONTROL",
            "rate-control requires a variant",
            "rate-control 23;",
        ),
        (
            "rate-control crf extra {}",
            "AUTHORING_OUTPUT_RATE_CONTROL",
            "rate-control accepts one variant",
            "rate-control crf extra {}",
        ),
        (
            "rate-control mystery;",
            "AUTHORING_OUTPUT_ENUM",
            "invalid rate-control variant 'mystery'",
            "mystery",
        ),
    ] {
        assert_diagnostic(rate_control, code, message, spanned_text);
    }
}

#[test]
fn crf_and_bitrate_reject_unknown_block_fields() {
    for (rate_control, message) in [
        (
            "rate-control crf { surprise 1; }",
            "crf rate control does not support `surprise`",
        ),
        (
            "rate-control bitrate { surprise 1; }",
            "bitrate rate control does not support `surprise`",
        ),
    ] {
        assert_diagnostic(rate_control, "AUTHORING_UNKNOWN_FIELD", message, "surprise");
    }
}
