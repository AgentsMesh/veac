use crate::authoring::{format_document, lower_document, parse};

const PROJECT: &str = r#"project audio {
  settings {
    timebase 1/1000;
    canvas 1920px by 1080px;
    frame-rate 30/1fps;
    sample-rate 48000hz;
  }
  entry sequence main;
  sequence main {
    layer audio dialogue {
      item voice {
        source generated silence;
        record { at 0s; duration 2s; }
        modifiers {
          audio clean {
            gain -3db;
            pan 0;
            muted false;
            normalize false;
            pitch preserve;
            processor eq tone-shaper {
              band presence { frequency 1000hz; gain -2db; q 0.7; }
            }
            processor high-pass cleanup-hpf { frequency 80hz; q 0.707; poles 2; }
            processor low-pass cleanup-lpf { frequency 16000hz; q 0.707; poles 2; }
            processor compressor dialogue-compressor {
              threshold -18db; ratio 4; attack 10ms; release 100ms;
              knee 6db; makeup-gain 3db; mix 100%;
            }
            processor limiter final-limiter { ceiling -1db; attack 1ms; release 50ms; }
            processor gate noise-gate {
              threshold -50db; ratio 2; attack 5ms; release 100ms; range -40db;
            }
            processor loudness target-loudness {
              integrated -14lufs; true-peak -1dbtp; range 7lu;
            }
            crossfade { fade-in 100ms; fade-out 100ms; curve equal-power; }
          }
        }
      }
    }
  }
}"#;

#[test]
fn audio_pipeline_parse_format_lower_and_validate() {
    let document = parse(PROJECT).unwrap();
    let formatted = format_document(&document);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    let envelope = lower_document(&document).unwrap();
    let audio = envelope.project.sequences[0].tracks[0].clips[0]
        .audio
        .as_ref()
        .unwrap();
    assert_eq!(audio.processors.len(), 7);
    assert_eq!(audio.processors[0].id.as_str(), "aud_tone-shaper");
    let veac_ir::AudioProcessorKind::ParametricEq { bands } = &audio.processors[0].kind else {
        panic!("first processor should be parametric EQ")
    };
    assert_eq!(bands[0].id.as_str(), "eqb_presence");
    assert!(audio.crossfade.is_some());
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn processor_and_eq_band_ids_are_required_and_owner_unique() {
    for source in [
        PROJECT.replace("processor limiter final-limiter", "processor limiter"),
        PROJECT.replace("processor low-pass cleanup-lpf", "processor low-pass cleanup-hpf"),
        PROJECT.replace(
            "band presence { frequency 1000hz; gain -2db; q 0.7; }",
            "band presence { frequency 1000hz; gain -2db; q 0.7; }\n              band presence { frequency 2000hz; gain 1db; q 1; }",
        ),
    ] {
        assert!(parse(&source).is_err());
    }
}

#[test]
fn unknown_audio_processor_is_rejected() {
    let source = PROJECT.replace("processor eq tone-shaper {", "processor mystery unknown {");
    let diagnostics = parse(&source).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_UNKNOWN_AUDIO_PROCESSOR"));
}

#[test]
fn audio_modifier_requires_audio_source() {
    let source = PROJECT
        .replace("layer audio dialogue", "layer visual dialogue")
        .replace("source generated silence", "source generated transparent");
    let diagnostics = lower_document(&parse(&source).unwrap()).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_LOWER_AUDIO_SOURCE"));
}

#[test]
fn bus_routing_and_sidechain_lower_as_typed_relations() {
    let source = r#"project routing {
  settings {
    timebase 1/1000; canvas 1920px by 1080px;
    frame-rate 30/1fps; sample-rate 48000hz;
  }
  entry sequence main;
  sequence main {
    layer audio music {
      route bus music-key;
      item key {
        source generated silence;
        record { at 0s; duration 2s; }
      }
    }
    layer audio dialogue {
      item voice {
        source generated silence;
        record { at 0s; duration 2s; }
      }
    }
    relation sidechain duck {
      endpoints { key bus music-key; target item voice; }
      dynamics {
        threshold -24db; ratio 4; attack 20ms; release 120ms;
      }
      timing { active { at 500ms; duration 1s; } }
    }
  }
}"#;
    let document = parse(source).unwrap();
    let formatted = format_document(&document);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    let envelope = lower_document(&document).unwrap();
    assert_eq!(envelope.project.relations.len(), 1);
    let veac_ir::RelationKind::Sidechain {
        key, parameters, ..
    } = &envelope.project.relations[0].kind
    else {
        panic!("expected sidechain relation")
    };
    assert_eq!(parameters.active_range.as_ref().unwrap().start.value, 500);
    assert!(matches!(key, veac_ir::RelationEndpoint::Bus { .. }));
    veac_ir::validate(&envelope).unwrap();
}
