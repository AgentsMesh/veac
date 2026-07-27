use crate::authoring::{lower_document, parse, Diagnostics};

use super::project;

fn source(slot: &str) -> String {
    let material = if slot.contains("accepts image;") {
        "image-default"
    } else {
        "video-default"
    };
    project(&format!(
        r#"resource video video-default {{ locator local {{ path "video.mov"; }} streams {{ video auto; audio disabled; }} }}
resource image image-default {{ locator local {{ path "image.png"; }} }}
sequence main {{
  layer visual title {{
    item hero {{
      source media resource {material};
      {slot}
      record {{ at 0s; duration 2s; }}
    }}
  }}
}}"#
    ))
}

fn assert_code(error: &Diagnostics, code: &str) {
    assert!(
        error.as_slice().iter().any(|value| value.code == code),
        "missing {code}: {error}"
    );
}

#[test]
fn every_template_slot_kind_and_fill_mode_lowers_exactly() {
    use veac_ir::{FillMode as F, SlotKind as K};
    for (accepts, kind) in [
        ("video", K::Video),
        ("image", K::Image),
        ("video-or-image", K::VideoOrImage),
    ] {
        for (fill, expected_fill) in [
            ("fit-duration", F::FitDuration),
            ("take-head", F::TakeHead),
            ("take-center", F::TakeCenter),
        ] {
            let authored = format!(
                "template-slot media {{ label \"Hero media\"; accepts {accepts}; fill {fill}; }}"
            );
            let document = parse(&source(&authored)).unwrap_or_else(|error| panic!("{error}"));
            let envelope = lower_document(&document).unwrap_or_else(|error| panic!("{error}"));
            let slot = envelope.project.sequences[0].tracks[0].clips[0]
                .replaceable
                .as_ref()
                .unwrap();
            assert_eq!((slot.kind, slot.fill), (kind, expected_fill));
            assert_eq!(slot.label, "Hero media");
            assert!(slot.min_source_duration.is_none());
        }
    }
}

#[test]
fn template_slot_parser_rejects_unknown_duplicate_and_non_block_forms() {
    for (slot, code) in [
        (
            "template-slot media { label \"x\"; accepts mystery; fill fit-duration; }",
            "AUTHORING_TEMPLATE_SLOT_VALUE",
        ),
        (
            "template-slot media { label \"x\"; accepts video; fill mystery; }",
            "AUTHORING_TEMPLATE_SLOT_VALUE",
        ),
        (
            "template-slot media { label \"x\"; accepts video; fill fit-duration; bogus true; }",
            "AUTHORING_UNKNOWN_FIELD",
        ),
        (
            "template-slot media { label \"x\"; label \"y\"; accepts video; fill fit-duration; }",
            "AUTHORING_DUPLICATE_FIELD",
        ),
        ("template-slot media;", "AUTHORING_EXPECTED_TOKEN"),
    ] {
        assert_code(&parse(&source(slot)).unwrap_err(), code);
    }
}

#[test]
fn template_slot_validation_reports_each_required_field_and_bad_duration() {
    for (body, message) in [
        ("accepts video; fill fit-duration;", "label"),
        ("label \"x\"; fill fit-duration;", "accepts"),
        ("label \"x\"; accepts video;", "fill"),
    ] {
        let error = parse(&source(&format!("template-slot media {{ {body} }}"))).unwrap_err();
        assert_code(&error, "AUTHORING_REQUIRED_FIELD");
        assert!(error.as_slice()[0].message.contains(message));
    }
    let invalid_duration = parse(&source(
        "template-slot media { label \"x\"; accepts video; fill fit-duration; min-source-duration 0s; }",
    ))
    .unwrap_err();
    assert_code(&invalid_duration, "AUTHORING_UNKNOWN_FIELD");
}
