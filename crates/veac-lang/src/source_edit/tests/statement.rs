use std::collections::BTreeMap;

use super::*;

const SOURCE: &str = "fn amount() -> time { let offset = 1s; offset }";

#[test]
fn statement_contract_accepts_one_complete_typed_statement() {
    for source in [
        "let offset = 2s;",
        "var offset: time = 2s;",
        "set offset = offset + 1s;",
    ] {
        assert_eq!(
            validate_source_edit_contract(&statement_batch(source)),
            Ok(())
        );
    }
}

#[test]
fn statement_contract_rejects_invalid_paths_targets_and_fragments() {
    let mut invalid_path = statement_batch("let offset = 2s;");
    let SourceEditOperation::SetStatement { site, .. } = &mut invalid_path.operations[0] else {
        unreachable!()
    };
    *site = StatementSite::BodyStatement {
        path: SourceExpressionPath::new(vec![SourceExpressionStep::BlockResult]),
    };
    assert_eq!(
        validate_source_edit_contract(&invalid_path),
        Err(SourceEditError::InvalidStatementPath)
    );

    let mut incompatible = statement_batch("let offset = 2s;");
    let SourceEditOperation::SetStatement { target, .. } = &mut incompatible.operations[0] else {
        unreachable!()
    };
    *target = SourceNodeRef::constant("main.veac", "amount");
    assert_eq!(
        validate_source_edit_contract(&incompatible),
        Err(SourceEditError::IncompatibleStatementSite)
    );

    for source in ["let offset = 2s", "offset + 1s", "let a = 1s; let b = 2s;"] {
        assert!(matches!(
            validate_source_edit_contract(&statement_batch(source)),
            Err(SourceEditError::InvalidStatement(_))
        ));
    }
    assert!(matches!(
        validate_source_edit_contract(&statement_batch("let offset = \0;")),
        Err(SourceEditError::InvalidStatement(_))
    ));
}

#[test]
fn statement_preconditions_compare_exact_authored_source() {
    let index = crate::program::SourceIndex::build(&sources()).unwrap();
    let site = statement_site(&index);
    let mut batch = statement_batch_with_site("let offset = 2s;", site.clone());
    batch
        .preconditions
        .push(SourcePrecondition::StatementEquals {
            target: target(),
            site: site.clone(),
            statement: StatementSource {
                source: "let offset = 1s;".to_owned(),
            },
        });
    batch.base_revision = index.revision().clone();
    assert_eq!(
        validate_source_edit_batch(&batch, index.revision(), &index),
        Ok(())
    );
    let SourcePrecondition::StatementEquals { statement, .. } = &mut batch.preconditions[0] else {
        unreachable!()
    };
    statement.source = "let offset = 1s ;".to_owned();
    assert_eq!(
        validate_source_edit_batch(&batch, index.revision(), &index),
        Err(SourceEditError::PreconditionFailed { index: 0 })
    );
}

#[test]
fn statement_range_resolution_uses_the_complete_validated_fragment() {
    let batch = statement_batch("let offset = 2s;");
    let replacement =
        resolve_source_edit_text(3, &batch.operations[0], TextRange { start: 10, end: 28 })
            .unwrap();
    assert_eq!(replacement.operation_index, 3);
    assert_eq!(replacement.target, target());
    assert_eq!(replacement.edit.range, TextRange { start: 10, end: 28 });
    assert_eq!(replacement.edit.replacement, "let offset = 2s;");
}

fn statement_batch(source: &str) -> SourceEditBatch {
    statement_batch_with_site(
        source,
        statement_site(&crate::program::SourceIndex::build(&sources()).unwrap()),
    )
}

fn statement_batch_with_site(source: &str, site: StatementSite) -> SourceEditBatch {
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_statement_edit").unwrap(),
        revision('a'),
    );
    batch.operations.push(SourceEditOperation::SetStatement {
        target: target(),
        site,
        statement: StatementSource {
            source: source.to_owned(),
        },
    });
    batch
}

fn statement_site(index: &crate::program::SourceIndex) -> StatementSite {
    index
        .inventory()
        .nodes
        .into_iter()
        .find(|node| node.target == target())
        .unwrap()
        .statements[0]
        .site
        .clone()
}

fn target() -> SourceNodeRef {
    SourceNodeRef::function("main.veac", "amount")
}

fn sources() -> BTreeMap<String, String> {
    BTreeMap::from([("main.veac".to_owned(), SOURCE.to_owned())])
}
