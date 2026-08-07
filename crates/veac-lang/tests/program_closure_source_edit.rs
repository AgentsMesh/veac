use std::fs;

use tempfile::tempdir;
use veac_lang::program::{apply_executable_source_edit_path, build_path, BuiltProgram};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef, SourcePrecondition,
};

#[path = "program_functions/support.rs"]
mod support;

const DECLARATIONS: &str = r#"fn selected() -> fn(time) -> time effect pure {
  fn(value: time) -> time effect pure { value + 1s }
}"#;

#[test]
fn editing_a_closure_factory_body_reexecutes_every_call_site() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let original_source = source();
    fs::write(&entry, &original_source).unwrap();
    let built = build_path(&entry).unwrap();
    assert_eq!(durations(&built), ["2s", "3s"]);

    let index = built.source_index().unwrap();
    let target = SourceNodeRef::function("main.veac", "selected");
    let original = BodySource {
        source: "{\n  fn(value: time) -> time effect pure { value + 1s }\n}".into(),
    };
    assert_eq!(
        index.body(&target, BodySite::FunctionBody).unwrap().source,
        original.source
    );
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_closure_function_body").unwrap(),
        index.revision().clone(),
    );
    batch.preconditions.push(SourcePrecondition::BodyEquals {
        target: target.clone(),
        site: BodySite::FunctionBody,
        body: original,
    });
    batch.operations.push(SourceEditOperation::SetBody {
        target,
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ fn(value: time) -> time effect pure { value + 2s } }".into(),
        },
    });

    let preview = apply_executable_source_edit_path(&entry, &batch).unwrap();
    assert_eq!(durations(&preview.built), ["3s", "4s"]);
    assert!(preview.source().unwrap().contains("value + 2s"));
    assert_eq!(fs::read_to_string(&entry).unwrap(), original_source);
    assert_ne!(preview.previous_revision, preview.new_revision);
    support::validate_canonical(&preview.built);
}

fn source() -> String {
    support::project_with_durations(DECLARATIONS, &["selected()(1s)", "selected()(2s)"])
}

fn durations(program: &BuiltProgram) -> [String; 2] {
    std::array::from_fn(|clip| support::item_duration(program, 0, 0, clip))
}
