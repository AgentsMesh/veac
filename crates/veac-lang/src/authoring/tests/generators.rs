use crate::authoring::{format_document, lower_document, parse};

const SETTINGS: &str = r#"settings {
  timebase 1/1000;
  canvas 1920px by 1080px;
  frame-rate 30/1fps;
  sample-rate 48000hz;
}"#;

#[test]
fn generator_algebra_round_trips_and_lowers_every_variant() {
    let source = format!(
        r#"project generators {{
  {SETTINGS}
  entry sequence main;
  sequence main {{
    layer visual graphics {{
      item linear {{
        source generated gradient linear {{
          from 0% 25%; to 100% 75%;
          stop 0% #112233ff; stop 45% #4488ccff; stop 100% #ffeeccff;
        }}
        record {{ at 0s; duration 1s; }}
      }}
      item radial {{
        source generated gradient radial {{
          center 50% 50%; radius 70%;
          stop 0% #ffffffff; stop 100% #00000000;
        }}
        record {{ at 1s; duration 1s; }}
      }}
      item rounded {{
        source generated shape {{
          geometry rounded-rectangle {{ bounds 5% 10% 90% 80%; radius 4%; }}
          fill gradient linear {{
            from 0% 0%; to 100% 100%;
            stop 0% #2255aaff; stop 50% #22aa88ff; stop 100% #112244ff;
          }}
          stroke 3px solid #ffffffff;
        }}
        record {{ at 2s; duration 1s; }}
      }}
      item polygon {{
        source generated shape {{
          geometry polygon {{ point 50% 5%; point 95% 90%; point 5% 90%; }}
          fill solid #ffcc00ff;
        }}
        record {{ at 3s; duration 1s; }}
      }}
      item path {{
        source generated shape {{
          geometry path {{ move 10% 10%; line 90% 10%; line 50% 90%; close; }}
          stroke 6px gradient radial {{
            center 50% 50%; radius 50%;
            stop 0% #ffffffff; stop 100% #ff0066ff;
          }}
        }}
        record {{ at 4s; duration 1s; }}
      }}
    }}
  }}
}}"#
    );
    let document = parse(&source).unwrap();
    let formatted = format_document(&document);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    let envelope = lower_document(&document).unwrap();
    assert_eq!(envelope.project.sequences[0].tracks[0].clips.len(), 5);
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn generator_variants_reject_unknown_and_duplicate_fields_early() {
    let source = format!(
        r#"project generators {{
  {SETTINGS}
  entry sequence main;
  sequence main {{
    layer visual graphics {{
      item invalid {{
        source generated gradient linear {{
          from 0% 0%; from 10% 10%; to 100% 100%; radius 30%;
          stop 0% #ffffffff; stop 100% #000000ff;
        }}
        record {{ at 0s; duration 1s; }}
      }}
    }}
  }}
}}"#
    );
    let diagnostics = parse(&source).unwrap_err();
    let codes = diagnostics
        .as_slice()
        .iter()
        .map(|value| value.code)
        .collect::<Vec<_>>();
    assert!(codes.contains(&"AUTHORING_DUPLICATE_FIELD"));
    assert!(codes.contains(&"AUTHORING_GENERATOR_FIELD"));
}

#[test]
fn unknown_generator_kind_is_rejected() {
    let source = format!(
        r#"project generators {{
  {SETTINGS}
  entry sequence main;
  sequence main {{
    layer visual graphics {{
      item invalid {{
        source generated mystery;
        record {{ at 0s; duration 1s; }}
      }}
    }}
  }}
}}"#
    );
    let diagnostics = parse(&source).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_GENERATOR_KIND"));
}
