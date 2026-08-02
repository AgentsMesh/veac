use std::fs;

use tempfile::tempdir;
use veac_lang::authoring::SourceDecl;
use veac_lang::program::{apply_source_edit_path, compile_path};
use veac_lang::source_edit::{
    ExpressionSite, ExpressionSource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

const MODULE: &str = r#"module {
  export const length title_size = 48px;
  export preset text-style heading {
    font family "Arial";
    size ${title_size};
    fill #ffffffff;
  }
  export component sequence title_card {
    param text title;
    param time duration default 300ms;
    slot visual backdrop;
    body {
      layer visual @background {
        item @backdrop {
          source slot backdrop;
          record { at 0s; duration ${duration}; }
        }
      }
      layer visual @copy {
        item @title {
          source text {
            content ${title};
            style { use text-style heading; }
            layout {
              box-width 500px; box-height 100px; wrap word; overflow clip;
              horizontal-align center; vertical-align middle;
            }
          }
          record { at 0s; duration ${duration}; }
        }
      }
    }
  }
}
"#;

const ENTRY: &str = r#"import "./brand.veac" as brand;
const time card_duration = 200ms + 100ms;
instance sequence opener from brand.title_card {
  bind title "Reusable title";
  bind duration card_duration;
  fill backdrop { source generated solid { color #17324dff; } }
}
project language-e2e {
  settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main;
  sequence main {
    layer visual content {
      item nested {
        source sequence sequence opener;
        record { at 0s; duration 300ms; }
      }
    }
  }
  delivery preview {
    sequence main;
    raster { canvas 640px by 360px; frame-rate 30fps; captions discard; }
    artifact video preview {
      target file "preview.mp4";
      mux mp4 {
        layout standard;
        video h264 {
          pixel-format yuv420p; alpha opaque; color-space source;
          rate-control crf { value 23; } gop automatic;
          b-frames automatic; profile automatic; level automatic;
        }
        audio none; passes single; accelerator auto;
      }
    }
  }
}
"#;

#[test]
fn modules_components_presets_slots_and_expressions_lower_to_canonical_ir() {
    let temp = fixture();
    let entry = temp.path().join("main.veac");
    let compiled = compile_path(&entry).unwrap();
    assert!(compiled.expanded_source().contains("sequence opener"));
    assert!(compiled
        .expanded_source()
        .contains("item veac-h-6-opener-5-title"));
    assert!(compiled.expanded_source().contains("size 48px"));
    assert_eq!(
        compiled
            .provenance()
            .get_local("opener", "title")
            .unwrap()
            .path,
        "brand.veac"
    );

    let envelope = veac_lang::authoring::lower_document(compiled.document()).unwrap();
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn source_edit_changes_only_the_target_expression_and_revalidates_the_graph() {
    let temp = fixture();
    let entry = temp.path().join("main.veac");
    let compiled = compile_path(&entry).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_source_edit_e2e").unwrap(),
        compiled.source_index().unwrap().revision().clone(),
    );
    batch.operations.push(SourceEditOperation::SetExpression {
        target: SourceNodeRef::constant("main.veac", "card_duration"),
        site: ExpressionSite::ConstantValue,
        expression: ExpressionSource {
            source: "400ms".into(),
        },
    });
    let preview = apply_source_edit_path(&entry, &batch).unwrap();
    assert_eq!(preview.source(), ENTRY.replace("200ms + 100ms", "400ms"));
    assert_ne!(preview.previous_revision, preview.new_revision);
}

#[test]
fn checked_in_example_forwards_slots_and_completes_the_source_edit_loop() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let entry = root.join("examples/programming-language/main.veac");
    let module = root.join("examples/programming-language/brand.veac");
    let original_entry = fs::read_to_string(&entry).unwrap();
    let original_module = fs::read_to_string(&module).unwrap();
    let compiled = compile_path(&entry).unwrap();
    assert!(!compiled.expanded_source().contains("source slot"));
    for instance in ["first-card", "second-card"] {
        let child_id = format!("veac-h-{}-{instance}-7-visuals", instance.len());
        let sequences = &compiled.document().project.sequences;
        let child = sequences
            .iter()
            .find(|value| value.id.value == child_id)
            .unwrap();
        assert_eq!(child.layers.len(), 2);
        assert!(matches!(
            child.layers[0].items[0].source,
            SourceDecl::Generated { .. }
        ));
        assert_eq!(child.layers[0].items[0].record.duration.raw, "3s");
        assert_eq!(child.layers[1].items[0].record.duration.raw, "1s");
        let parent = sequences
            .iter()
            .find(|value| value.id.value == instance)
            .unwrap();
        let SourceDecl::Sequence { sequence, .. } = &parent.layers[0].items[0].source else {
            panic!("title card must reference its nested visuals");
        };
        assert_eq!(sequence.id.value, child_id);
        assert!(compiled
            .provenance()
            .get_local_path(instance, &["visuals", "backdrop"])
            .is_some());
    }
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_checked_in_language_example").unwrap(),
        compiled.source_index().unwrap().revision().clone(),
    );
    batch.operations.push(SourceEditOperation::SetExpression {
        target: SourceNodeRef::constant("main.veac", "section_duration"),
        site: ExpressionSite::ConstantValue,
        expression: ExpressionSource {
            source: "6s".into(),
        },
    });

    let preview = apply_source_edit_path(&entry, &batch).unwrap();
    assert_eq!(
        preview.source(),
        original_entry.replacen("brand.card_duration", "6s", 1)
    );
    assert_eq!(preview.compiled.sources()["brand.veac"], original_module);
    assert_eq!(fs::read_to_string(&entry).unwrap(), original_entry);
    assert_eq!(fs::read_to_string(&module).unwrap(), original_module);
    let envelope = veac_lang::authoring::lower_document(preview.compiled.document()).unwrap();
    veac_ir::validate(&envelope).unwrap();
}

fn fixture() -> tempfile::TempDir {
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("main.veac"), ENTRY).unwrap();
    fs::write(temp.path().join("brand.veac"), MODULE).unwrap();
    temp
}
