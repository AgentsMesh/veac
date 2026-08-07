use super::*;

fn source_with(declarations: &str, duration: &str) -> String {
    format!(
        "{declarations}\n{}",
        GENERATED_SOURCE.replace("during(0s, 200ms)", &format!("during(0s, {duration})"))
    )
}

#[test]
fn check_executes_all_closed_collection_operations() {
    let declarations = r#"fn aggregate() -> time {
  let mapped = map(0 .. 3, fn(value: int) -> time effect pure { 100ms });
  let selected = filter(mapped, fn(value: time) -> bool effect pure { value >= 100ms });
  let shifted = for value in selected { value + 10ms };
  fold(shifted, 0ms, fn(total: time, value: time) -> time effect pure { total + value })
}"#;
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &source_with(declarations, "aggregate()"));
    veac()
        .args(["check", source.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Executable source is valid"));
}

#[test]
fn check_executes_function_values_transport_through_collections() {
    let declarations = r#"fn aggregate() -> time {
  let callbacks = map([1], fn(value: int) -> fn(int) -> int effect pure effect pure {
    fn(next: int) -> int effect pure { next + value }
  });
  let result = fold(
    callbacks,
    0,
    fn(total: int, callback: fn(int) -> int effect pure) -> int effect pure {
      total + callback(41)
    }
  );
  if result == 42 { 200ms } else { 300ms }
}"#;
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &source_with(declarations, "aggregate()"));
    veac()
        .args(["check", source.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Executable source is valid"));
}

#[test]
fn source_edit_reexecutes_an_authored_collection_function() {
    let declarations = r#"fn aggregate() -> time {
  let values = for value in [100ms, 100ms] { value };
  fold(values, 0ms, fn(total: time, value: time) -> time effect pure { total + value })
}"#;
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &source_with(declarations, "aggregate()"));
    let target = SourceNodeRef::function("main.veac", "aggregate");
    let prepared = veac_lang::program::prepare_path(&source).unwrap();
    let index = prepared.source_index().unwrap();
    let original = index.body(&target, BodySite::FunctionBody).unwrap();
    let replacement = original
        .source
        .replace("100ms, 100ms", "100ms, 100ms, 100ms");
    let mut edit = SourceEditBatch::new(
        veac_ir::OperationId::new("op_cli_collection_body").unwrap(),
        revision(&source),
    );
    edit.operations.push(SourceEditOperation::SetBody {
        target,
        site: BodySite::FunctionBody,
        body: BodySource {
            source: replacement,
        },
    });
    let batch_path = temp.path().join("collection-edit.json");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&edit).unwrap(),
    )
    .unwrap();

    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
        ])
        .assert()
        .success();
    veac()
        .args(["check", source.to_str().unwrap()])
        .assert()
        .success();
    assert!(std::fs::read_to_string(source)
        .unwrap()
        .contains("100ms, 100ms, 100ms"));
}
