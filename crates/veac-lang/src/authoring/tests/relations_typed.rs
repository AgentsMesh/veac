use crate::authoring::{format_document, parse, RelationKind, SignalEndpoint, TransitionStyle};

fn project(relations: &str) -> String {
    format!(
        r#"project typed-relations {{
  entry sequence main;
  settings {{
    timebase 1/1000000;
    canvas 1920px by 1080px;
    frame-rate 30fps;
    sample-rate 48000hz;
  }}
  sequence main {{
{relations}
  }}
}}"#
    )
}

pub(super) const ALL_RELATIONS: &str = r#"
    relation transition intro-cut {
      endpoints { from item first; to item second; }
      timing { duration 300ms; alignment centered; }
      style { wipe { direction left; angle 0deg; softness 0.2; } }
    }
    relation matte title-matte {
      endpoints { producer item matte-source; consumer item title; }
      style { luma { invert true; } }
    }
    relation sidechain voice-duck {
      endpoints { key bus dialogue; target item music; }
      dynamics { threshold -18db; ratio 4; attack 20ms; release 250ms; }
      timing { active { at 500ms; duration 2s; } }
    }
    relation group linked-title {
      members { item title; item matte-source; }
    }
    relation av-link camera-sync {
      endpoints { video item camera; audio { item boom; item lav; } }
    }
"#;

#[test]
fn all_relation_kinds_have_typed_ast_nodes() {
    let document = parse(&project(ALL_RELATIONS)).unwrap();
    let structures = &document.project.sequences[0].structures;
    assert_eq!(structures.len(), 5);
    let RelationKind::Transition(value) = relation(structures, 0) else {
        panic!("expected transition")
    };
    assert_eq!(value.endpoints.from.id.value, "first");
    assert!(matches!(value.style, TransitionStyle::Wipe { .. }));
    let RelationKind::Sidechain(value) = relation(structures, 2) else {
        panic!("expected sidechain")
    };
    assert!(matches!(value.endpoints.key, SignalEndpoint::Bus { .. }));
    assert_eq!(value.timing.active.as_ref().unwrap().at.raw, "500ms");
    assert!(matches!(relation(structures, 1), RelationKind::Matte(_)));
    assert!(matches!(relation(structures, 3), RelationKind::Group(_)));
    assert!(matches!(relation(structures, 4), RelationKind::AvLink(_)));
}

#[test]
fn canonical_relation_format_is_idempotent() {
    let document = parse(&project(ALL_RELATIONS)).unwrap();
    let once = format_document(&document);
    let twice = format_document(&parse(&once).unwrap());
    assert_eq!(once, twice);
    assert!(once.contains("relation transition intro-cut {\n      endpoints {"));
    assert!(once.contains("style {\n        wipe {"));
    assert!(once.contains("key bus dialogue;"));
    assert!(!once.contains("type wipe;"));
}

#[test]
fn every_transition_style_variant_is_closed_and_canonical() {
    let styles = [
        "dissolve;",
        "fade { color black; }",
        "wipe { direction right; angle 0deg; softness 0.1; }",
        "slide { direction up; amount 1; }",
        "zoom { direction in; amount 1.2; }",
        "circle { direction close; softness 0.3; }",
        "pixelize { amount 12; }",
    ];
    for (index, style) in styles.iter().enumerate() {
        let relation = format!(
            "relation transition t{index} {{ endpoints {{ from item a; to item b; }} \
             timing {{ duration 1s; alignment after-cut; }} style {{ {style} }} }}"
        );
        let document = parse(&project(&relation)).unwrap();
        let formatted = format_document(&document);
        assert_eq!(formatted, format_document(&parse(&formatted).unwrap()));
    }
}

#[test]
fn relation_schema_errors_are_structured() {
    let cases = [
        ("relation mystery x {}", "AUTHORING_UNKNOWN_RELATION_KIND"),
        (
            "relation group x { members { item a; item b; } mystery true; }",
            "AUTHORING_UNKNOWN_FIELD",
        ),
        (
            "relation transition x { endpoints { from track a; to item b; } timing { duration 1s; alignment centered; } style { dissolve; } }",
            "AUTHORING_RELATION_ENDPOINT_TYPE",
        ),
        (
            "relation transition x { endpoints { from item a; to item b; } timing { duration \"1s\"; alignment centered; } style { dissolve; } }",
            "AUTHORING_FIELD_TYPE",
        ),
        (
            "relation transition x { endpoints { from item a; to item b; } timing { duration 1s; alignment centered; } style { spin; } }",
            "AUTHORING_UNKNOWN_TRANSITION_STYLE",
        ),
        (
            "relation sidechain x { endpoints { key item voice; target item music; } dynamics { threshold -18db; ratio 4; attack 1ms; release 1ms; } }",
            "AUTHORING_RELATION_ENDPOINT_TYPE",
        ),
    ];
    for (source, expected) in cases {
        let diagnostics = parse(&project(source)).unwrap_err();
        assert!(
            diagnostics
                .as_slice()
                .iter()
                .any(|diagnostic| diagnostic.code == expected),
            "expected {expected}, got {diagnostics:?}"
        );
    }
}

fn relation(structures: &[crate::authoring::StructureDecl], index: usize) -> &RelationKind {
    let crate::authoring::StructureDecl::Relation(value) = &structures[index] else {
        panic!("expected relation")
    };
    &value.kind
}
