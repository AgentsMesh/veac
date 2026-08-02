use super::{entry, error_code};
use crate::authoring::lower_document;
use crate::program::compile_source;

#[test]
fn fractional_scalars_and_lengths_materialize_as_core_literals() {
    let declarations = r#"const length third = 1px / 3;
const scalar ratio = 1 / 3;
component sequence fractional {
  param identifier content_id;
  body { layer visual @graphics { item ${content_id} {
    source generated solid { color #112233ff; }
    record { at 0s; duration 1s; }
    modifiers {
      layout bounds { frame { width ${third}; height 10px; fit fill; } }
      effect sharpen { type video.sharpen; parameter amount ${ratio}; }
    }
  } } }
}
instance sequence card from fractional {
  bind content_id identifier("fractional-item");
}"#;
    let body = r#"sequence main { layer visual content { item nested {
    source sequence sequence card; record { at 0s; duration 1s; }
  } } }"#;
    let compiled = compile_source(&entry(declarations, body)).unwrap();
    assert!(compiled.expanded_source().contains("item fractional-item"));
    assert!(!compiled.expanded_source().contains(" / 3"));
    let envelope = lower_document(compiled.document()).unwrap();
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn rational_that_rounds_to_one_does_not_become_integer_syntax() {
    let source = entry(
        concat!(
            "const scalar almost_one = 1 - 1 / ",
            "170141183460469231731687303715884105727;"
        ),
        r#"sequence main { layer visual content {
    order ${almost_one};
  } }"#,
    );
    let compiled = compile_source(&source).unwrap();
    assert!(compiled.expanded_source().contains("order 1.0;"));
    let errors = lower_document(compiled.document()).unwrap_err();
    assert_eq!(errors.as_slice()[0].code, "AUTHORING_LOWER_INTEGER");
}

#[test]
fn non_decimal_exact_time_fails_at_the_typed_materialization_boundary() {
    let source = entry(
        "const time duration = 1s / 3;",
        r#"sequence main { layer visual content { item sample {
    source generated transparent; record { at 0s; duration ${duration}; }
  } } }"#,
    );
    assert_eq!(error_code(&source), "PROGRAM_EXPRESSION_MATERIALIZATION");
}

#[test]
fn expression_and_core_strings_share_unicode_escape_semantics() {
    let source = entry(
        r#"const text label = "A\u{1b}B";"#,
        r#"sequence main { layer visual content { item sample {
    source text { content ${label}; } record { at 0s; duration 1s; }
  } } }"#,
    );
    let compiled = compile_source(&source).unwrap();
    let formatted = crate::authoring::format_document(compiled.document());
    assert!(formatted.contains(r#"content "A\u{1b}B";"#));
    assert!(crate::authoring::parse(&formatted).is_ok());
}

#[test]
fn core_strings_reject_unknown_escapes_instead_of_changing_text() {
    let source = entry(
        "",
        r#"sequence main { layer visual content { item sample {
    source text { content "bad\qescape"; } record { at 0s; duration 1s; }
  } } }"#,
    );
    let errors = crate::authoring::parse(&source).unwrap_err();
    assert_eq!(errors.as_slice()[0].code, "AUTHORING_LEX_STRING_ESCAPE");
}
