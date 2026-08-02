use std::fs;

use tempfile::tempdir;
use veac_lang::program::{apply_source_edit_path, compile_path, SourceTransactionError};
use veac_lang::source_edit::{
    ExpressionSite, ExpressionSource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
    SourceSnapshot,
};

const SOURCE: &str = r#"project source-index-edit {
  settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main;
  resource image logo { locator local { path "old.png"; } }
  sequence main {
    layer visual content {
      item title {
        source text { content "旧标题"; }
        record { at 0s; duration 1s; }
        state { playback disabled; }
        modifiers {
          effect title-sharpen { type video.sharpen; parameter amount 1; }
        }
      }
      item secondary {
        source text { content "保留标题"; }
        record { at 2s; duration 1s; }
        modifiers {
          layout title-sharpen {
            placement anchor { at center; inset { x 0px; y 0px; } }
            frame { width 320px; height 180px; fit contain; }
          }
        }
      }
    }
  }
}"#;

#[test]
fn edits_project_sites_and_recompiles_the_source_of_truth() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, SOURCE).unwrap();
    let compiled = compile_path(&entry).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_project_sites").unwrap(),
        compiled.source_index().unwrap().revision().clone(),
    );
    batch.operations = vec![
        operation(item("title"), ExpressionSite::ItemRecordStart, "250ms"),
        operation(item("title"), ExpressionSite::ItemRecordDuration, "2s"),
        operation(item("title"), ExpressionSite::TextContent, "\"新标题\""),
        operation(item("title"), ExpressionSite::ItemEnabled, "enabled"),
        operation(
            SourceNodeRef::resource("main.veac", "logo"),
            ExpressionSite::ResourceLocator,
            "\"new.png\"",
        ),
        operation(
            modifier("title", "title-sharpen"),
            ExpressionSite::ModifierParameter {
                parameter: "amount".into(),
            },
            "2",
        ),
    ];
    let preview = apply_source_edit_path(&entry, &batch).unwrap();
    let expected = SOURCE
        .replace("\"old.png\"", "\"new.png\"")
        .replace("\"旧标题\"", "\"新标题\"")
        .replace("at 0s; duration 1s", "at 250ms; duration 2s")
        .replace("playback disabled", "playback enabled")
        .replace("parameter amount 1", "parameter amount 2");
    assert_eq!(preview.source(), expected);
    assert!(preview.source().contains("layout title-sharpen"));
    assert!(preview.source().contains("width 320px"));
    veac_ir::validate(&veac_lang::authoring::lower_document(preview.compiled.document()).unwrap())
        .unwrap();
}

#[test]
fn item_enabled_edit_is_checked_against_the_playback_closed_set() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, SOURCE).unwrap();
    let compiled = compile_path(&entry).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_invalid_playback").unwrap(),
        compiled.source_index().unwrap().revision().clone(),
    );
    batch.operations.push(operation(
        item("title"),
        ExpressionSite::ItemEnabled,
        "true",
    ));
    assert!(matches!(
        apply_source_edit_path(&entry, &batch),
        Err(SourceTransactionError::Program(_))
    ));
}

#[test]
fn example_revisions_accept_locally_reused_modifier_ids() {
    let generated = example_index("generated-graphics");
    assert!(generated.node_exists(&modifier_at(
        "generated-graphics",
        "dark-top-left",
        "top-left-cell",
        "cell",
    )));
    assert!(generated.node_exists(&modifier_at(
        "generated-graphics",
        "dark-bottom-right",
        "bottom-right-cell",
        "cell",
    )));

    let blends = example_index("blend-modes");
    assert!(blends.node_exists(&modifier_at("blend-modes", "overlays", "screen", "mode")));
    assert!(blends.node_exists(&modifier_at("blend-modes", "overlays", "multiply", "mode")));

    let effects = example_index("video-effects");
    assert!(effects.node_exists(&modifier_at(
        "video-effects",
        "effects",
        "focus",
        "canvas-fill"
    )));
    assert!(effects.node_exists(&modifier_at(
        "video-effects",
        "effects",
        "finish",
        "canvas-fill"
    )));
}

fn operation(target: SourceNodeRef, site: ExpressionSite, source: &str) -> SourceEditOperation {
    SourceEditOperation::SetExpression {
        target,
        site,
        expression: ExpressionSource {
            source: source.into(),
        },
    }
}

fn item(id: &str) -> SourceNodeRef {
    SourceNodeRef::item("main.veac", "source-index-edit", "main", "content", id)
}

fn modifier(item: &str, id: &str) -> SourceNodeRef {
    SourceNodeRef::modifier(
        "main.veac",
        "source-index-edit",
        "main",
        "content",
        item,
        id,
    )
}

fn example_index(example: &str) -> veac_lang::program::SourceIndex {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples")
        .join(example)
        .join("main.veac");
    compile_path(&path).unwrap().source_index().unwrap()
}

fn modifier_at(project: &str, layer: &str, item: &str, modifier: &str) -> SourceNodeRef {
    SourceNodeRef::modifier("main.veac", project, "main", layer, item, modifier)
}
