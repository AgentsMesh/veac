use crate::authoring::{format_document, lower_document, parse};

const PROJECT: &str = r#"project template {
  settings {
    timebase 1/1000;
    canvas 1920px by 1080px;
    frame-rate 30fps;
    sample-rate 48000hz;
  }
  entry sequence main;

  resource video hero-placeholder {
    locator local { path "placeholder.mov"; }
    streams { video auto; audio disabled; }
  }

  sequence main {
    layer video media-slots {
      item hero {
        source media resource hero-placeholder;
        record { at 0s; duration 6s; }
        mapping linear { from 0s; to 6s; }
        template-slot media {
          accepts video;
          fill fit-duration;
          label "Hero media";
          minimum-source-duration 2s;
        }
      }
    }

    layer visual titles {
      item title {
        source text { content "Original title"; }
        record { at 0s; duration 6s; }
        template-slot text;
      }
    }
  }
}"#;

#[test]
fn item_owned_template_slots_round_trip_and_lower() {
    let document = parse(PROJECT).unwrap();
    let formatted = format_document(&document);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    let envelope = lower_document(&document).unwrap();
    let sequence = &envelope.project.sequences[0];
    let media = sequence
        .tracks
        .iter()
        .find(|track| track.id.as_str() == "trk_media-slots")
        .unwrap();
    let slot = media.clips[0].replaceable.as_ref().unwrap();
    assert_eq!(slot.label, "Hero media");
    assert_eq!(slot.kind, veac_ir::SlotKind::Video);
    let title = sequence
        .tracks
        .iter()
        .find(|track| track.id.as_str() == "trk_titles")
        .unwrap();
    assert!(title.clips[0].template_editable_text);
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn template_slot_source_kind_is_checked_early() {
    let invalid = PROJECT.replace(
        "source media resource hero-placeholder;",
        "source generated solid { color #000000ff; }",
    );
    let diagnostics = parse(&invalid).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_TEMPLATE_MEDIA_SOURCE"));
}

#[test]
fn placeholder_resource_has_single_item_ownership() {
    let duplicate = r#"
      item duplicate {
        source media resource hero-placeholder;
        record { at 7s; duration 1s; }
      }
    "#;
    let replacement = format!("      }}{duplicate}    }}\n\n    layer visual titles");
    let invalid = PROJECT.replace("      }\n    }\n\n    layer visual titles", &replacement);
    let diagnostics = parse(&invalid).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_TEMPLATE_SLOT_OWNERSHIP"));
}
