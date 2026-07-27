use crate::authoring::{format_document, lower_document, parse};
use veac_ir::{ApplyOperation, ApplyTarget};

pub(super) const SOURCE: &str = r#"project apply-test {
  settings {
    timebase 1/1000; canvas 1920px by 1080px;
    frame-rate 30/1fps; sample-rate 48000hz;
  }
  entry sequence main;
  sequence nested {
    layer visual elsewhere {
      item remote { source generated transparent; record { at 0s; duration 2s; } }
    }
  }
  sequence main {
    layer visual lower {
      order 0;
      item alpha { source generated transparent; record { at 0s; duration 2s; } }
      item gamma { source generated transparent; record { at 0s; duration 2s; } }
    }
    layer visual upper {
      order 1;
      item beta { source generated transparent; record { at 0s; duration 2s; } }
    }
    layer audio sound {
      order 2;
      item audio { source generated silence; record { at 0s; duration 2s; } }
    }
    relation group duo { members { item alpha; item beta; } }
    apply band-grade {
      scope composite-band { from layer lower; through layer upper; }
      record { at 0s; duration 2s; }
      pipeline {
        stage effect tone {
          type video.color_adjust; enabled false;
          record { at 100ms; duration 200ms; }
          parameter contrast 1.1;
        }
        stage color grade {
          input-space { primaries bt709; transfer bt709; matrix bt709; range limited; }
          working-space { primaries bt709; transfer linear; matrix rgb; range full; }
          output-space { primaries bt709; transfer bt709; matrix bt709; range limited; }
          basic {
            exposure 0stops; temperature 6500k; tint 0;
            highlights 0; shadows 0; fade 0%;
          }
        }
      }
      mix {
        opacity 80%; blend screen;
        mask window { shape circle; feather 2px; }
      }
    }
    apply layer-grade {
      scope layer upper;
      record { at 0s; duration 1s; }
      pipeline { stage effect layer-fx { type video.color_adjust; } }
      mix {}
    }
    apply selected-grade {
      scope items { item gamma; group duo; }
      record { at 0s; duration 1s; }
      pipeline { stage effect selected-fx { type video.color_adjust; } }
      mix {}
    }
  }
}"#;

#[test]
fn apply_syntax_formats_idempotently_and_lowers_without_synthetic_timeline_items() {
    let document = parse(SOURCE).unwrap();
    let formatted = format_document(&document);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    assert!(formatted.contains("scope composite-band"));
    let tone = formatted.find("stage effect tone").unwrap();
    let grade = formatted.find("stage color grade").unwrap();
    assert!(tone < grade);

    let envelope = lower_document(&document).unwrap();
    let sequence = &envelope.project.sequences[1];
    assert_eq!(sequence.tracks.len(), 3);
    assert_eq!(
        sequence
            .tracks
            .iter()
            .map(|track| track.clips.len())
            .sum::<usize>(),
        4
    );
    assert_eq!(sequence.applies.len(), 3);
    assert!(matches!(
        &sequence.applies[0].target,
        ApplyTarget::CompositeBand { from_track_id, through_track_id }
            if from_track_id.as_str() == "trk_lower" && through_track_id.as_str() == "trk_upper"
    ));
    assert!(matches!(
        &sequence.applies[1].target,
        ApplyTarget::Layer { track_id } if track_id.as_str() == "trk_upper"
    ));
    let ApplyTarget::ItemSet { item_ids } = &sequence.applies[2].target else {
        panic!("item set expected")
    };
    assert_eq!(
        item_ids.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
        ["itm_alpha", "itm_beta", "itm_gamma"]
    );
    let band = &sequence.applies[0];
    assert_eq!(band.stages[0].id.as_str(), "aps_tone");
    assert_eq!(band.stages[1].id.as_str(), "aps_grade");
    assert!(!band.stages[0].enabled);
    assert!(band.stages[0].active_range.is_some());
    let ApplyOperation::Effect { effect } = &band.stages[0].operation else {
        panic!("effect stage expected")
    };
    assert!(effect.enabled && effect.enable_range.is_none());
    assert!(matches!(
        band.stages[1].operation,
        ApplyOperation::Color { .. }
    ));
    assert_eq!(band.mix.masks.len(), 1);
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn item_scopes_reject_duplicates_non_visual_and_cross_sequence_targets() {
    for (needle, replacement, code) in [
        (
            "item gamma; group duo;",
            "item alpha; group duo;",
            "AUTHORING_LOWER_APPLY_DUPLICATE_ITEM",
        ),
        (
            "item gamma; group duo;",
            "item audio;",
            "AUTHORING_LOWER_APPLY_NON_VISUAL_ITEM",
        ),
        (
            "item gamma; group duo;",
            "item remote;",
            "AUTHORING_LOWER_APPLY_TARGET",
        ),
        (
            "item gamma; group duo;",
            "group absent;",
            "AUTHORING_LOWER_APPLY_TARGET",
        ),
    ] {
        let document = parse(&SOURCE.replace(needle, replacement)).unwrap();
        let diagnostics = lower_document(&document).unwrap_err();
        assert!(diagnostics
            .as_slice()
            .iter()
            .any(|value| value.code == code));
    }
}

#[test]
fn composite_band_rejects_reversed_boundaries() {
    let source = SOURCE.replace(
        "from layer lower; through layer upper;",
        "from layer upper; through layer lower;",
    );
    let diagnostics = lower_document(&parse(&source).unwrap()).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_LOWER_APPLY_BAND_ORDER"));
}
