use std::fs;

use tempfile::tempdir;
use veac_lang::authoring::lower_document;
use veac_lang::program::{apply_source_edit_path, compile_path};
use veac_lang::source_edit::{
    ExpressionSite, ExpressionSource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

const SOURCE: &str = r#"component sequence card {
  param time duration default 1s;
  body { layer visual @content { item @panel {
    source generated transparent; record { at 0s; duration ${duration}; }
  } } }
}
instance sequence default-card from card {}
instance sequence bound-card from card { bind duration 2s; }
project component-edits {
  settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main;
  sequence main {}
}"#;

#[test]
fn component_defaults_and_instance_arguments_are_real_source_edit_sites() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, SOURCE).unwrap();
    let compiled = compile_path(&entry).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_component_parameters").unwrap(),
        compiled.source_index().unwrap().revision().clone(),
    );
    batch.operations = vec![
        operation(
            SourceNodeRef::component("main.veac", "card"),
            ExpressionSite::ComponentParameterDefault {
                parameter: "duration".into(),
            },
            "1500ms",
        ),
        operation(
            SourceNodeRef::component_instance("main.veac", "bound-card"),
            ExpressionSite::ComponentInstanceArgument {
                parameter: "duration".into(),
            },
            "3s",
        ),
    ];

    let preview = apply_source_edit_path(&entry, &batch).unwrap();
    assert_eq!(
        preview.source(),
        SOURCE
            .replace("default 1s", "default 1500ms")
            .replace("bind duration 2s", "bind duration 3s")
    );
    let expanded = preview.compiled.expanded_source();
    assert!(expanded.contains("duration 1500ms;"));
    assert!(expanded.contains("duration 3s;"));
    let envelope = lower_document(preview.compiled.document()).unwrap();
    veac_ir::validate(&envelope).unwrap();
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
