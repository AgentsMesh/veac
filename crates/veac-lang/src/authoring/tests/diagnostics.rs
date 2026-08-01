use super::project;
use crate::authoring::parse;

#[test]
fn rejects_non_normative_declarations_and_missing_fields() {
    let cases = [
        (
            project("resource video bad { uri \"bad.mov\"; }"),
            "AUTHORING_RESOURCE_FIELD",
        ),
        (
            project("resource image bad { locator local {} }"),
            "AUTHORING_REQUIRED_FIELD",
        ),
        (
            project("sequence main { layer overlay bad {} }"),
            "AUTHORING_LAYER_TYPE",
        ),
        (
            project("sequence main { layer video v { item x { source generated transparent; } } }"),
            "AUTHORING_REQUIRED_FIELD",
        ),
        (
            project(
                "sequence main { apply grade { scope items { item missing; } record { at 0s; duration 1s; } } }",
            ),
            "AUTHORING_REQUIRED_FIELD",
        ),
    ];
    assert_codes(cases);
}

#[test]
fn rejects_invalid_literals_references_and_mapping_order() {
    let invalid_frame_rate = project("").replace("30000/1001fps", "30");
    let invalid_color = project(
        "sequence main { layer visual v { item x { source generated solid { color #12345; } record { at 0s; duration 1s; } } } }",
    );
    let missing_resource = project(
        r#"
  sequence main {
    layer video v {
      item x {
        source media resource missing;
        record { at 0s; duration 1s; }
      }
    }
  }
"#,
    );
    let key_order = project(
        r#"
  sequence main {
    layer visual v {
      item x {
        source generated solid { color #202020ff; }
        record { at 0s; duration 1s; }
        mapping curve {
          key later { at 1s; source 0s; interpolation linear; }
          key earlier { at 0s; source 1s; interpolation linear; }
        }
      }
    }
  }
"#,
    );
    assert_codes([
        (invalid_frame_rate, "AUTHORING_FRAME_RATE_LITERAL"),
        (invalid_color, "AUTHORING_LEX_COLOR"),
        (missing_resource, "AUTHORING_REFERENCE_NOT_FOUND"),
        (key_order, "AUTHORING_MAPPING_KEY_ORDER"),
    ]);
}

#[test]
fn diagnostics_include_stable_codes_and_byte_spans() {
    let errors = parse("project demo { settings { canvas nope; } }").unwrap_err();
    assert!(!errors.as_slice().is_empty());
    assert!(errors
        .as_slice()
        .iter()
        .all(|error| error.code.starts_with("AUTHORING_") && error.span.end >= error.span.start));
}

#[test]
fn project_entry_is_required_unique_typed_and_resolved() {
    let valid = project("sequence main {}");
    let cases = [
        (
            valid.replace("  entry sequence main;\n", ""),
            "AUTHORING_REQUIRED_FIELD",
        ),
        (
            valid.replace(
                "  entry sequence main;",
                "  entry sequence main; entry sequence main;",
            ),
            "AUTHORING_DUPLICATE_FIELD",
        ),
        (
            valid.replace("entry sequence main", "entry item main"),
            "AUTHORING_ENTRY_KIND",
        ),
        (
            valid.replace("entry sequence main", "entry sequence missing"),
            "AUTHORING_ENTRY_NOT_FOUND",
        ),
    ];
    assert_codes(cases);
}

#[test]
fn legacy_output_declarations_are_rejected() {
    let declaration = ["out", "put"].concat();
    let target = ["file", "-name"].concat();
    for kind in [
        "video",
        "image-sequence",
        "caption-sidecar",
        "audio-stem",
        "scope",
    ] {
        let source = project(&format!(
            "sequence main {{}} {declaration} {kind} bad {{ sequence main; {target} \"bad.mp4\"; encoding {{}} }}"
        ));
        let errors = parse(&source).unwrap_err();
        assert!(errors
            .as_slice()
            .iter()
            .any(|error| error.code == "AUTHORING_PROJECT_MEMBER"));
    }
}

fn assert_codes<const N: usize>(cases: [(String, &str); N]) {
    for (source, code) in cases {
        let errors = parse(&source).unwrap_err();
        assert!(
            errors.as_slice().iter().any(|error| error.code == code),
            "expected {code}, got {:?}",
            errors.as_slice()
        );
    }
}
