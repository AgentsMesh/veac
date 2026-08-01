use veac_ir::{ClipSource, ProjectEnvelope, SourceTimeMap};
use veac_lang::{format_document, lower_document, parse};

#[test]
fn source_mapping_variants_survive_public_round_trip() {
    let cases = [
        (
            "mapping linear { from 0s; to 10s; outside hold-both; }",
            "linear",
        ),
        (
            "mapping curve { key p0 { at 0s; source 0s; interpolation linear; } \
             key p1 { at 5s; source 7s; interpolation linear; } outside hold-last; }",
            "curve",
        ),
        ("mapping freeze { source 2s; }", "freeze"),
    ];

    for (mapping, expected) in cases {
        let project = compile_round_trip(&mapped_project(mapping)).unwrap();
        let clip = &project.project.sequences[0].tracks[0].clips[0];
        if expected == "freeze" {
            assert!(matches!(clip.source, ClipSource::FreezeFrame { .. }));
            continue;
        }
        let mapping = clip
            .source_mapping
            .as_ref()
            .unwrap_or_else(|| panic!("missing {expected} mapping"));
        match (expected, &mapping.time_map) {
            ("linear", SourceTimeMap::Linear { rate, .. }) => assert_eq!(rate.numerator, 2),
            ("curve", SourceTimeMap::Curve { segments, .. }) => assert_eq!(segments.len(), 1),
            _ => panic!("unexpected lowered mapping for {expected}"),
        }
    }
}

#[test]
fn interpolation_variants_survive_parameter_curve_round_trip() {
    let document = parse(INTERPOLATIONS).unwrap();
    let formatted = format_document(&document);
    for variant in [
        "interpolation hold;",
        "interpolation linear;",
        "interpolation ease-in;",
        "interpolation ease-out;",
        "interpolation ease-in-out;",
        "interpolation cubic-bezier {",
        "interpolation spring {",
    ] {
        assert!(formatted.contains(variant), "missing `{variant}`");
    }
    let reparsed = parse(&formatted).unwrap();
    lower_document(&reparsed).unwrap();
}

#[test]
fn mappings_and_interpolations_reject_incomplete_or_unknown_forms() {
    for invalid in [
        "mapping warp { at 1s; }",
        "mapping freeze {}",
        "mapping linear { from 0s; to 5s; outside random; }",
        "mapping curve { key p0 { at 0s; } key p1 { at 5s; source 5s; } }",
    ] {
        assert!(compile_round_trip(&mapped_project(invalid)).is_err());
    }

    let unknown = INTERPOLATIONS.replace("interpolation hold;", "interpolation bounce;");
    assert!(compile_round_trip(&unknown).is_err());
    let incomplete =
        INTERPOLATIONS.replace("x1 0.25; y1 0.1; x2 0.25; y2 1.0;", "x1 0.25; y1 0.1;");
    assert!(compile_round_trip(&incomplete).is_err());
}

fn compile_round_trip(source: &str) -> Result<ProjectEnvelope, String> {
    let document = parse(source).map_err(|error| format!("{error:?}"))?;
    let formatted = format_document(&document);
    let reparsed = parse(&formatted).map_err(|error| format!("{error:?}"))?;
    lower_document(&reparsed).map_err(|error| format!("{error:?}"))
}

fn mapped_project(mapping: &str) -> String {
    format!(
        r#"project closed-mapping {{
settings {{ timebase 1/600; canvas 1920px by 1080px; frame-rate 24/1fps;
  sample-rate 48000hz; }}
resource video footage {{
  locator local {{ path "video.mp4"; }}
  streams {{ video auto; audio disabled; }}
}}
entry sequence main;
sequence main {{
  layer video base {{
    item mapped {{
      source media resource footage;
      record {{ at 0s; duration 5s; }}
      {mapping}
    }}
  }}
}}
}}"#
    )
}

const INTERPOLATIONS: &str = r#"project closed-interpolation {
settings { timebase 1/600; canvas 1920px by 1080px; frame-rate 24/1fps;
  sample-rate 48000hz; }
entry sequence main;
sequence main {
  layer visual base {
    item animated {
      source generated transparent;
      record { at 0s; duration 6s; }
      modifiers {
        transform motion {
          position curve {
            key k0 { at 0s; value { x 0px; y 0px; } interpolation hold; }
            key k1 { at 1s; value { x 10px; y 10px; } interpolation linear; }
            key k2 { at 2s; value { x 20px; y 20px; } interpolation ease-in; }
            key k3 { at 3s; value { x 30px; y 30px; } interpolation ease-out; }
            key k4 { at 4s; value { x 40px; y 40px; } interpolation ease-in-out; }
            key k5 { at 5s; value { x 50px; y 50px; }
              interpolation cubic-bezier { x1 0.25; y1 0.1; x2 0.25; y2 1.0; }
            }
            key k6 { at 6s; value { x 60px; y 60px; }
              interpolation spring { frequency 1.5; decay 6; initial-velocity 0; }
            }
          }
        }
      }
    }
  }
}
}"#;
