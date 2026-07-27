use crate::authoring::{format_document, lower_document, parse};

pub(super) const PROJECT: &str = r#"project multicam-demo {
  settings {
    timebase 1/1000;
    canvas 1920px by 1080px;
    frame-rate 30/1fps;
    sample-rate 48000hz;
  }
  entry sequence main;

  resource video host-video {
    locator local { path "host.mov"; }
    streams { video auto; audio auto; }
  }
  resource video guest-video {
    locator local { path "guest.mov"; }
    streams { video auto; audio auto; }
  }

  multicam interview {
    sync audio { reference angle host; }
    angle host {
      source resource host-video;
      source-offset 0s;
    }
    angle guest {
      source resource guest-video;
      source-offset 120ms;
    }
  }

  sequence main {
    layer video program {
      item interview-cut {
        source multicam multicam interview {
          switch angle host { at 0s; duration 4s; }
          switch angle guest { at 4s; duration 3s; }
        }
        record { at 10s; duration 7s; }
      }
    }
  }
}"#;

#[test]
fn multicam_entity_and_switch_program_round_trip_and_lower() {
    let document = parse(PROJECT).unwrap();
    let formatted = format_document(&document);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    let envelope = lower_document(&document).unwrap();
    assert_eq!(envelope.project.multicam_groups.len(), 1);
    let group = &envelope.project.multicam_groups[0];
    assert_eq!(group.id.as_str(), "mcg_interview");
    assert_eq!(group.angles[0].id.as_str(), "ang_guest");
    let source = &envelope.project.sequences[0].tracks[0].clips[0].source;
    let veac_ir::ClipSource::Multicam { switches, .. } = source else {
        panic!("multicam source expected")
    };
    assert_eq!(switches.len(), 2);
    assert_eq!(switches[1].range.start.value, 4_000);
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn multicam_partition_is_checked_in_authoring_time() {
    let source = PROJECT.replace(
        "switch angle guest { at 4s; duration 3s; }",
        "switch angle guest { at 5s; duration 2s; }",
    );
    let diagnostics = parse(&source).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_MULTICAM_PARTITION"));
}

#[test]
fn multicam_rejects_unknown_angles_and_leaf_sources() {
    let source = PROJECT.replace("reference angle host", "reference angle missing");
    let diagnostics = parse(&source).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_MULTICAM_REFERENCE"));

    let leaf = PROJECT.replace(
        "source multicam multicam interview {\n          switch angle host { at 0s; duration 4s; }\n          switch angle guest { at 4s; duration 3s; }\n        }",
        "source multicam multicam interview;",
    );
    assert!(parse(&leaf).is_err());
}
