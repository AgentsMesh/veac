use crate::authoring::{format_document, lower_document, parse, AnnotationPayloadDecl};

use super::project;

const PROVENANCE: &str = r#"provenance {
  producer "test";
  request-sha256 "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
  response-sha256 "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
}"#;

#[test]
fn parses_and_formats_every_closed_annotation_kind() {
    let body = format!(
        r#"
  sequence main {{
    layer visual canvas {{
      item hero {{
        source generated transparent;
        record {{ at 0s; duration 4s; }}
      }}
    }}
  }}
  annotation marker marker-a {{
    target project; span untimed;
    payload {{ label "Opening"; color #ffcc00ff; }} {PROVENANCE}
  }}
  annotation language language-a {{
    target sequence main; span point {{ at 0s; }}
    payload {{ candidate "en" {{ confidence 0.9; }} }} {PROVENANCE}
  }}
  annotation scene-boundary boundary-a {{
    target item hero; span point {{ at 1s; }}
    payload {{ confidence 0.8; hard-cut true; }} {PROVENANCE}
  }}
  annotation scene scene-a {{
    target item hero; span range {{ at 0s; duration 2s; }}
    payload {{}} {PROVENANCE}
  }}
  annotation beat beat-a {{
    target sequence main; span point {{ at 2s; }}
    payload {{ confidence 0.95; bar 1; beat-in-bar 2; tempo-bpm 120; meter 4; }}
    {PROVENANCE}
  }}
  annotation silence silence-a {{
    target item hero; span range {{ at 2s; duration 500ms; }}
    payload {{ mean-db -54db; confidence 0.91; }} {PROVENANCE}
  }}
  annotation filler filler-a {{
    target item hero; span range {{ at 2.5s; duration 200ms; }}
    payload {{ token "um"; confidence 0.7; suggestion tighten; }} {PROVENANCE}
  }}
  annotation highlight highlight-a {{
    target item hero; span range {{ at 0s; duration 4s; }}
    payload {{ score 0.88; rationale "Strong hook"; evidence "fast opening"; }}
    {PROVENANCE}
  }}
  annotation review review-a {{
    target item hero; span range {{ at 0s; duration 4s; }}
    payload {{ action keep; rationale "Approved"; confidence 0.99; }} {PROVENANCE}
  }}
"#
    );
    let document = parse(&project(&body)).unwrap();
    assert_eq!(document.project.annotations.len(), 9);
    assert!(matches!(
        document.project.annotations[0].payload,
        AnnotationPayloadDecl::Marker(_)
    ));
    let formatted = format_document(&document);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    let diagnostics = lower_document(&document).unwrap_err();
    assert!(!diagnostics.as_slice().is_empty());
}

#[test]
fn marker_lowers_to_canonical_annotation() {
    let source = format!(
        r#"project demo {{
  settings {{
    timebase 1/1000; canvas 1920px by 1080px;
    frame-rate 30/1fps; sample-rate 48000hz;
  }}
  entry sequence main;
  sequence main {{
    layer visual canvas {{
      item hero {{
        source generated transparent;
        record {{ at 0s; duration 2s; }}
      }}
    }}
  }}
  annotation marker intro {{
    target project; span untimed;
    payload {{ label "Opening"; }} {PROVENANCE}
  }}
}}"#
    );
    let envelope = lower_document(&parse(&source).unwrap()).unwrap();
    assert_eq!(envelope.project.annotations.len(), 1);
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn rejects_unknown_annotation_fields_and_bad_hashes() {
    let source = project(
        r#"annotation marker bad {
  target project; span untimed;
  payload { label "Bad"; mystery true; }
  provenance {
    producer "test"; request-sha256 "BAD";
    response-sha256 "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
  }
}"#,
    );
    let diagnostics = parse(&source).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_UNKNOWN_FIELD"));
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_INVALID_SHA256"));
}
