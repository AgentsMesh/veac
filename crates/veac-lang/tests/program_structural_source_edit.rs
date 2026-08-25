use std::fs;

use tempfile::tempdir;
use veac_lang::program::{apply_executable_source_edit_path, build_path, BuiltProgram};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef, SourcePrecondition,
};

#[path = "program_functions/support.rs"]
mod support;

const DECLARATIONS: &str = r#"fn reorder(values: list<int>) -> list<int> { values }
fn choose(values: list<int>) -> time {
  if reorder(values) == [1, 2, 3] { 1s } else { 2s }
}"#;

#[test]
fn editing_a_structural_function_body_reexecutes_every_call_site() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let original_source = source();
    fs::write(&entry, &original_source).unwrap();
    let built = build_path(&entry).unwrap();
    assert_eq!(durations(&built), ["1s", "1s"]);

    let index = built.source_index().unwrap();
    let target = SourceNodeRef::function("main.veac", "reorder");
    let original = BodySource {
        source: "{ values }".into(),
    };
    assert_eq!(
        index.body(&target, BodySite::FunctionBody).unwrap().source,
        original.source
    );
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_structural_function_body").unwrap(),
        built.source_revision().unwrap(),
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
            source: "{ [3, 2, 1] }".into(),
        },
    });

    let preview = apply_executable_source_edit_path(&entry, &batch).unwrap();
    assert_eq!(durations(&preview.built), ["2s", "2s"]);
    assert!(preview.source().unwrap().contains("{ [3, 2, 1] }"));
    assert_eq!(fs::read_to_string(&entry).unwrap(), original_source);
    assert_ne!(preview.previous_revision, preview.new_revision);
    support::validate_canonical(&preview.built);
}

fn source() -> String {
    support::project_with_durations(DECLARATIONS, &["choose([1, 2, 3])", "choose([1, 2, 3])"])
}

fn durations(program: &BuiltProgram) -> [String; 2] {
    std::array::from_fn(|clip| support::item_duration(program, 0, 0, clip))
}
