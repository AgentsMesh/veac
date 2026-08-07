use super::*;

fn source() -> String {
    format!(
        "fn offset(value: time) -> time {{ value + 1s }}\n{}",
        GENERATED_SOURCE.replace("during(0s, 200ms)", "during(0s, offset(1s))")
    )
}

#[test]
fn source_edit_command_applies_a_typed_function_body() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &source());
    let target = SourceNodeRef::function("main.veac", "offset");
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_cli_function_body").unwrap(),
        revision(&source),
    );
    batch.preconditions.push(SourcePrecondition::BodyEquals {
        target: target.clone(),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ value + 1s }".into(),
        },
    });
    batch.operations.push(SourceEditOperation::SetBody {
        target,
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ value + 2s }".into(),
        },
    });
    let batch_path = temp.path().join("function-body.json");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&batch).unwrap(),
    )
    .unwrap();

    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"modules\": ["))
        .stdout(predicate::str::contains("\"main.veac\""));
    assert!(std::fs::read_to_string(source)
        .unwrap()
        .contains("{ value + 2s }"));
}
