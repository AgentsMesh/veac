use super::project;
use crate::authoring::{parse, InterpolationKind, MappingDecl, SourceOutOfRangeDecl};

fn document(mapping: &str) -> String {
    project(&format!(
        r#"
  resource video media {{
    locator local {{ path "shot.mov"; }}
    streams {{ video auto; audio disabled; }}
  }}
  sequence main {{
    layer video picture {{
      item shot {{
        source media resource media;
        record {{ at 0s; duration 10s; }}
        {mapping}
      }}
    }}
  }}
"#
    ))
}

fn parsed(mapping: &str) -> MappingDecl {
    let document = parse(&document(mapping)).expect("mapping should parse");
    let clip = &document.project.sequences[0].layers[0].items[0];
    clip.mapping.clone().expect("expected mapping")
}

fn codes(mapping: &str) -> Vec<&'static str> {
    parse(&document(mapping))
        .expect_err("mapping should fail")
        .as_slice()
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

#[test]
fn mapping_defaults_are_exact() {
    let MappingDecl::Linear {
        from, to, outside, ..
    } = parsed("mapping linear { from 1s; to 2s; }")
    else {
        panic!("expected linear mapping")
    };
    assert_eq!((from.raw.as_str(), to.raw.as_str()), ("1s", "2s"));
    assert_eq!(outside, SourceOutOfRangeDecl::Strict);

    let MappingDecl::Freeze { source, .. } = parsed("mapping freeze { source 3s; }") else {
        panic!("expected freeze mapping")
    };
    assert_eq!(source.raw, "3s");

    let MappingDecl::Curve { keys, outside, .. } =
        parsed("mapping curve { key only { at 0s; source 0s; interpolation linear; } }")
    else {
        panic!("expected curve mapping")
    };
    assert_eq!(keys.len(), 1);
    assert_eq!(outside, SourceOutOfRangeDecl::Strict);
}

#[test]
fn curve_mapping_preserves_every_leaf_interpolation() {
    let MappingDecl::Curve { keys, .. } = parsed(
        r#"mapping curve {
          outside hold-both;
          key a { at 0s; source 0s; interpolation hold; }
          key b { at 1s; source 1s; interpolation linear; }
          key c { at 2s; source 2s; interpolation ease-in; }
          key d { at 3s; source 3s; interpolation ease-out; }
          key e { at 4s; source 4s; interpolation ease-in-out; }
        }"#,
    ) else {
        panic!("expected curve mapping")
    };
    assert_eq!(
        keys.iter()
            .map(|key| key.interpolation.kind.clone())
            .collect::<Vec<_>>(),
        vec![
            InterpolationKind::Hold,
            InterpolationKind::Linear,
            InterpolationKind::EaseIn,
            InterpolationKind::EaseOut,
            InterpolationKind::EaseInOut,
        ]
    );
}

#[test]
fn mapping_header_errors_are_exact() {
    assert_eq!(codes("mapping mystery {}"), ["AUTHORING_MAPPING_TYPE"]);
    assert_eq!(
        codes("mapping linear;"),
        ["AUTHORING_EXPECTED_TOKEN", "AUTHORING_ITEM_MEMBER"]
    );
}

#[test]
fn linear_mapping_reports_duplicate_unknown_and_missing_fields() {
    assert_eq!(
        codes("mapping linear { from 0s; from 1s; bogus 2s; }"),
        [
            "AUTHORING_DUPLICATE_FIELD",
            "AUTHORING_MAPPING_FIELD",
            "AUTHORING_REQUIRED_FIELD",
        ]
    );
}

#[test]
fn freeze_mapping_field_errors_are_exact() {
    assert_eq!(
        codes("mapping freeze { source 0s; source 1s; bogus 2s; }"),
        ["AUTHORING_DUPLICATE_FIELD", "AUTHORING_MAPPING_FIELD"]
    );
    assert_eq!(codes("mapping freeze {}"), ["AUTHORING_REQUIRED_FIELD"]);
}

#[test]
fn curve_mapping_reports_container_errors() {
    assert_eq!(
        codes("mapping curve { outside hold-both; outside hold-first; bogus 2s; }"),
        [
            "AUTHORING_DUPLICATE_FIELD",
            "AUTHORING_MAPPING_FIELD",
            "AUTHORING_MAPPING_KEYS",
            "AUTHORING_MAPPING_KEYS",
        ]
    );
}

#[test]
fn mapping_key_reports_duplicate_and_unknown_fields() {
    assert_eq!(
        codes(
            r#"mapping curve { key k {
              at 0s; at 1s;
              source 0s; source 1s;
              interpolation linear; interpolation hold;
              bogus 2s;
            } }"#,
        ),
        [
            "AUTHORING_DUPLICATE_FIELD",
            "AUTHORING_DUPLICATE_FIELD",
            "AUTHORING_DUPLICATE_FIELD",
            "AUTHORING_MAPPING_FIELD",
        ]
    );
}

#[test]
fn mapping_key_requires_each_field() {
    for (body, field) in [
        ("source 0s; interpolation linear;", "at"),
        ("at 0s; interpolation linear;", "source"),
        ("at 0s; source 0s;", "interpolation"),
    ] {
        let diagnostics = parse(&document(&format!(
            "mapping curve {{ key k {{ {body} }} }}"
        )))
        .expect_err("missing key field should fail");
        assert_eq!(diagnostics.as_slice()[0].code, "AUTHORING_REQUIRED_FIELD");
        assert_eq!(
            diagnostics.as_slice()[0].message,
            format!("mapping requires key {field}")
        );
    }
}

#[test]
fn mapping_key_body_and_interpolation_errors_are_exact() {
    assert_eq!(
        codes("mapping curve { key k; }"),
        [
            "AUTHORING_EXPECTED_TOKEN",
            "AUTHORING_MAPPING_FIELD",
            "AUTHORING_MAPPING_KEYS",
            "AUTHORING_MAPPING_KEYS",
        ]
    );
    assert_eq!(
        codes("mapping curve { key k { at 0s; source 0s; interpolation cubic-bezier; } }"),
        ["AUTHORING_INTERPOLATION"]
    );
}
