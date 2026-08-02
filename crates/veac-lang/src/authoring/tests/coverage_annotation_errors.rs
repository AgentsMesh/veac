use crate::authoring::{parse, Diagnostics};

use super::project;

const PROVENANCE: &str = r#"provenance {
  producer "coverage";
  request-sha256 "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
  response-sha256 "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
}"#;

fn source(annotation: &str) -> String {
    project(&format!(
        r#"resource image poster {{ locator local {{ path "poster.png"; }} }}
sequence main {{
  layer visual picture {{
    item blank {{
      source generated transparent;
      record {{ at 0s; duration 2s; }}
    }}
  }}
}}
{annotation}"#
    ))
}

fn marker(target: &str, span: &str) -> String {
    format!(
        "annotation marker note {{ target {target}; span {span} payload {{ label \"note\"; }} {PROVENANCE} }}"
    )
}

fn assert_code(error: &Diagnostics, code: &str) {
    assert!(
        error.as_slice().iter().any(|value| value.code == code),
        "missing {code}: {error}"
    );
}

#[test]
fn every_unknown_annotation_target_kind_reports_reference_diagnostics() {
    for target in [
        "sequence missing",
        "layer missing",
        "item missing",
        "resource missing",
        "multicam missing",
    ] {
        let error = parse(&source(&marker(target, "untimed;"))).unwrap_err();
        assert_code(&error, "AUTHORING_UNKNOWN_REFERENCE");
        assert!(error
            .as_slice()
            .iter()
            .any(|value| value.message.contains("missing")));
    }
}

#[test]
fn annotation_validation_rejects_duplicate_ids_and_invalid_times() {
    let duplicate = format!(
        "{}\n{}",
        marker("project", "untimed;"),
        marker("project", "untimed;")
    );
    assert_code(
        &parse(&source(&duplicate)).unwrap_err(),
        "AUTHORING_DUPLICATE_ID",
    );

    let negative = marker("project", "point { at -1s; }");
    assert_code(
        &parse(&source(&negative)).unwrap_err(),
        "AUTHORING_TIME_LITERAL",
    );
    let zero_duration = format!(
        "annotation filler note {{ target item blank; span range {{ at 0s; duration 0s; }} payload {{ token \"um\"; confidence 0.8; suggestion keep; }} {PROVENANCE} }}"
    );
    assert_code(
        &parse(&source(&zero_duration)).unwrap_err(),
        "AUTHORING_TIME_LITERAL",
    );
}

#[test]
fn annotation_parser_rejects_bad_target_span_and_incomplete_provenance_shapes() {
    let bad_target = marker("mystery missing", "untimed;");
    assert_code(
        &parse(&source(&bad_target)).unwrap_err(),
        "AUTHORING_INVALID_ANNOTATION_TARGET",
    );
    let bad_span = marker("project", "mystery;");
    assert_code(
        &parse(&source(&bad_span)).unwrap_err(),
        "AUTHORING_FIELD_SHAPE",
    );
    let missing_hash = r#"annotation marker note {
  target project; span untimed; payload { label "note"; }
  provenance { producer "coverage"; request-sha256 "a"; }
}"#;
    assert_code(
        &parse(&source(missing_hash)).unwrap_err(),
        "AUTHORING_REQUIRED_FIELD",
    );
}
