use veac_ir::ApplyTarget;
use veac_lang::authoring::{lower_document, parse};

const SOURCE: &str = r#"project apply-integration {
  settings {
    timebase 1/1000; canvas 1920px by 1080px;
    frame-rate 30/1fps; sample-rate 48000hz;
  }
  entry sequence main;
  sequence main {
    layer visual base {
      order 0;
      item hero { source generated transparent; record { at 0s; duration 2s; } }
    }
    layer visual overlay {
      order 1;
      item title { source generated transparent; record { at 0s; duration 2s; } }
    }
    relation group selection { members { item hero; item title; } }
    apply treatment {
      scope items { group selection; }
      record { at 0s; duration 2s; }
      pipeline {
        stage effect contrast {
          type video.color_adjust;
          parameter contrast 1.1;
        }
      }
      mix { opacity 90%; blend screen; }
    }
  }
}"#;

#[test]
fn public_authoring_pipeline_emits_only_first_class_apply_ir() {
    let document = parse(SOURCE).unwrap();
    let envelope = lower_document(&document).unwrap();
    let sequence = &envelope.project.sequences[0];
    assert_eq!(sequence.tracks.len(), 2);
    assert_eq!(
        sequence
            .tracks
            .iter()
            .map(|track| track.clips.len())
            .sum::<usize>(),
        2
    );
    assert_eq!(sequence.applies.len(), 1);
    assert!(matches!(
        &sequence.applies[0].target,
        ApplyTarget::ItemSet { item_ids }
            if item_ids.iter().map(|id| id.as_str()).collect::<Vec<_>>()
                == ["itm_hero", "itm_title"]
    ));
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn public_parser_rejects_the_legacy_generic_scope() {
    let legacy = SOURCE.replace(
        "scope items { group selection; }",
        "scope { group selection; }",
    );
    let diagnostics = parse(&legacy).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_APPLY_SCOPE_KIND"));
}
