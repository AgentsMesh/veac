use std::fs;

use tempfile::tempdir;
use veac_lang::program::{apply_executable_source_edit_path, build_path, BuiltProgram};
use veac_lang::source_edit::{
    SourceEditBatch, SourceEditOperation, SourceNodeRef, SourcePrecondition, StatementSite,
    StatementSource,
};

#[path = "program_functions/support.rs"]
mod support;

const DECLARATIONS: &str = r#"fn aggregate() -> time {
  let values = map(0 .. 3, fn(value: int) -> time effect pure { 100ms });
  let shifted = for value in values { value + 10ms };
  fold(shifted, 0ms, fn(total: time, value: time) -> time effect pure { total + value })
}"#;

#[test]
fn editing_a_collection_function_reexecutes_every_call_site() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let original_source = source();
    fs::write(&entry, &original_source).unwrap();
    let built = build_path(&entry).unwrap();
    assert_eq!(durations(&built), ["330ms", "330ms"]);

    let index = built.source_index().unwrap();
    let target = SourceNodeRef::function("main.veac", "aggregate");
    let (site, original) = index
        .inventory()
        .nodes
        .into_iter()
        .find(|node| node.target == target)
        .unwrap()
        .statements
        .into_iter()
        .find(|statement| statement.source.contains("let shifted ="))
        .map(|statement| (statement.site, statement.source))
        .unwrap();
    assert!(matches!(site, StatementSite::BodyStatement { .. }));
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_collection_function_body").unwrap(),
        index.revision().clone(),
    );
    batch
        .preconditions
        .push(SourcePrecondition::StatementEquals {
            target: target.clone(),
            site: site.clone(),
            statement: StatementSource { source: original },
        });
    batch.operations.push(SourceEditOperation::SetStatement {
        target,
        site,
        statement: StatementSource {
            source: "let shifted = for value in values { value + 20ms };".to_owned(),
        },
    });

    let preview = apply_executable_source_edit_path(&entry, &batch).unwrap();
    assert_eq!(durations(&preview.built), ["360ms", "360ms"]);
    assert!(preview
        .source()
        .unwrap()
        .contains("let shifted = for value in values { value + 20ms };"));
    assert_eq!(fs::read_to_string(&entry).unwrap(), original_source);
    support::validate_canonical(&preview.built);
}

#[test]
fn standalone_valid_statement_still_requires_a_valid_owner_scope() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let original_source = source();
    fs::write(&entry, &original_source).unwrap();
    let built = build_path(&entry).unwrap();
    let index = built.source_index().unwrap();
    let target = SourceNodeRef::function("main.veac", "aggregate");
    let statement = index
        .inventory()
        .nodes
        .into_iter()
        .find(|node| node.target == target)
        .unwrap()
        .statements
        .into_iter()
        .find(|statement| statement.source.contains("let shifted ="))
        .unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_collection_invalid_scope").unwrap(),
        index.revision().clone(),
    );
    batch.operations.push(SourceEditOperation::SetStatement {
        target,
        site: statement.site,
        statement: StatementSource {
            source: "let shifted = for value in missing { value + 20ms };".to_owned(),
        },
    });

    assert!(apply_executable_source_edit_path(&entry, &batch).is_err());
    assert_eq!(fs::read_to_string(&entry).unwrap(), original_source);
}

fn source() -> String {
    support::project_with_durations(DECLARATIONS, &["aggregate()", "aggregate()"])
}

fn durations(program: &BuiltProgram) -> [String; 2] {
    std::array::from_fn(|clip| support::item_duration(program, 0, 0, clip))
}
