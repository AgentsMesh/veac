use std::collections::{BTreeMap, BTreeSet};

use super::*;

#[derive(Default)]
struct Snapshot {
    nodes: BTreeSet<SourceNodeRef>,
    expressions: BTreeMap<(SourceNodeRef, ExpressionSite), String>,
    statements: BTreeMap<(SourceNodeRef, StatementSite), String>,
    bodies: BTreeMap<(SourceNodeRef, BodySite), String>,
    declarations: BTreeMap<(SourceNodeRef, DeclarationSite), String>,
}

impl SourceSnapshot for Snapshot {
    fn node_exists(&self, target: &SourceNodeRef) -> bool {
        self.nodes.contains(target)
    }

    fn expression_source(&self, target: &SourceNodeRef, site: &ExpressionSite) -> Option<&str> {
        self.expressions
            .get(&(target.clone(), site.clone()))
            .map(String::as_str)
    }

    fn statement_source(&self, target: &SourceNodeRef, site: &StatementSite) -> Option<&str> {
        self.statements
            .get(&(target.clone(), site.clone()))
            .map(String::as_str)
    }

    fn body_source(&self, target: &SourceNodeRef, site: BodySite) -> Option<&str> {
        self.bodies.get(&(target.clone(), site)).map(String::as_str)
    }

    fn declaration_source(&self, target: &SourceNodeRef, site: DeclarationSite) -> Option<&str> {
        self.declarations
            .get(&(target.clone(), site))
            .map(String::as_str)
    }
}

#[test]
fn valid_batch_checks_revision_and_preconditions() {
    let target = target();
    let site = ExpressionSite::ConstantValue;
    let mut snapshot = Snapshot::default();
    snapshot.nodes.insert(target.clone());
    snapshot
        .expressions
        .insert((target.clone(), site.clone()), "4s".to_owned());
    let function = SourceNodeRef::function("timeline/main.veac", "duration");
    snapshot.bodies.insert(
        (function.clone(), BodySite::FunctionBody),
        "{ 4s }".to_owned(),
    );
    let declaration_target =
        SourceNodeRef::struct_field("timeline/main.veac", "Timing", "duration");
    snapshot.declarations.insert(
        (
            declaration_target.clone(),
            DeclarationSite::StructFieldDeclaration,
        ),
        "duration: time".to_owned(),
    );
    let mut batch = batch();
    batch.preconditions = vec![
        SourcePrecondition::NodeExists {
            target: target.clone(),
        },
        SourcePrecondition::ExpressionEquals {
            target,
            site,
            expression: ExpressionSource {
                source: "4s".to_owned(),
            },
        },
        SourcePrecondition::BodyEquals {
            target: function,
            site: BodySite::FunctionBody,
            body: BodySource {
                source: "{ 4s }".to_owned(),
            },
        },
        SourcePrecondition::DeclarationEquals {
            target: declaration_target,
            site: DeclarationSite::StructFieldDeclaration,
            declaration: DeclarationSource {
                source: "duration: time".to_owned(),
            },
        },
    ];
    assert_eq!(
        validate_source_edit_batch(&batch, &revision('a'), &snapshot),
        Ok(())
    );
}

#[test]
fn stale_revision_wins_before_precondition_evaluation() {
    let mut batch = batch();
    batch
        .preconditions
        .push(SourcePrecondition::NodeExists { target: target() });
    assert!(matches!(
        validate_source_edit_batch(&batch, &revision('b'), &Snapshot::default()),
        Err(SourceEditError::StaleRevision { .. })
    ));
}

#[test]
fn failed_precondition_reports_its_index() {
    let mut batch = batch();
    batch.preconditions = vec![
        SourcePrecondition::NodeAbsent { target: target() },
        SourcePrecondition::NodeExists { target: target() },
    ];
    assert_eq!(
        validate_source_edit_batch(&batch, &revision('a'), &Snapshot::default()),
        Err(SourceEditError::PreconditionFailed { index: 1 })
    );
}

#[test]
fn contract_rejects_non_atomic_empty_and_incompatible_operations() {
    let mut value = batch();
    value.atomic = false;
    assert_eq!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::AtomicRequired)
    );
    value.atomic = true;
    value.operations.clear();
    assert_eq!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::EmptyOperations)
    );
    value.operations.push(SourceEditOperation::SetExpression {
        target: SourceNodeRef::function("timeline/main.veac", "duration"),
        site: ExpressionSite::ConstantValue,
        expression: ExpressionSource {
            source: "\"hello\"".to_owned(),
        },
    });
    assert_eq!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::IncompatibleExpressionSite)
    );
}

#[test]
fn contract_rejects_bad_schema_digest_target_and_expression() {
    let mut value = batch();
    value.schema = "other".to_owned();
    assert_eq!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::InvalidSchema)
    );
    value = batch();
    value.base_revision.authored_source_graph_sha256 = "ABC".to_owned();
    assert!(matches!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::InvalidDigest(_))
    ));
    value = batch();
    value.operations = vec![SourceEditOperation::SetExpression {
        target: SourceNodeRef {
            module: "../escape.veac".to_owned(),
            ..target()
        },
        site: ExpressionSite::ConstantValue,
        expression: ExpressionSource {
            source: "\0".to_owned(),
        },
    }];
    assert!(matches!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::InvalidModulePath(_))
    ));
}
