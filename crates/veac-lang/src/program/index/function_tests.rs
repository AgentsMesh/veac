use std::collections::BTreeMap;

use crate::source_edit::{
    apply_resolved_text_replacements, resolve_source_edit_text, BodySite, BodySource,
    SourceEditBatch, SourceEditOperation, SourceNodeRef, SourcePrecondition, SourceSnapshot,
};

const SOURCE: &str = r#"module {
  export fn twice(value: time) -> time {
    value * 2
  }
}"#;

#[test]
fn indexes_function_nodes_bodies_and_exact_source_ranges() {
    let index = super::SourceIndex::build_snapshot(&sources(SOURCE)).unwrap();
    let target = function();
    assert!(index.node_exists(&target));
    let body = index.body(&target, BodySite::FunctionBody).unwrap();
    assert_eq!(body.source, "{\n    value * 2\n  }");
    assert_eq!(&SOURCE[body.range.start..body.range.end], body.source);
}

#[test]
fn function_body_edit_round_trips_through_a_rebuilt_index() {
    let index = super::SourceIndex::build_snapshot(&sources(SOURCE)).unwrap();
    let revision = super::super::test_revision(&index);
    let operation = set_body("{ value * 2 + 250ms }");
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_function_body_index").unwrap(),
        revision.clone(),
    );
    batch.preconditions.push(SourcePrecondition::BodyEquals {
        target: function(),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{\n    value * 2\n  }".into(),
        },
    });
    batch.operations.push(operation.clone());
    crate::source_edit::validate_source_edit_batch(&batch, &revision, &index).unwrap();

    let range = index
        .body(&function(), BodySite::FunctionBody)
        .unwrap()
        .range;
    let replacement = resolve_source_edit_text(0, &operation, range).unwrap();
    let edited = apply_resolved_text_replacements("motion.veac", SOURCE, &[replacement]).unwrap();
    let rebuilt = super::SourceIndex::build_snapshot(&sources(&edited)).unwrap();
    let body = rebuilt.body(&function(), BodySite::FunctionBody).unwrap();

    assert_eq!(body.source, "{ value * 2 + 250ms }");
    assert_eq!(&edited[body.range.start..body.range.end], body.source);
    assert_ne!(rebuilt.revision(), index.revision());
}

#[test]
fn body_precondition_compares_the_complete_authored_block() {
    let index = super::SourceIndex::build_snapshot(&sources(SOURCE)).unwrap();
    let revision = super::super::test_revision(&index);
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_function_body_precondition").unwrap(),
        revision.clone(),
    );
    batch.preconditions.push(SourcePrecondition::BodyEquals {
        target: function(),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ value * 2 }".into(),
        },
    });
    batch.operations.push(set_body("{ value * 3 }"));
    assert!(matches!(
        crate::source_edit::validate_source_edit_batch(&batch, &revision, &index),
        Err(crate::source_edit::SourceEditError::PreconditionFailed { index: 0 })
    ));
}

fn function() -> SourceNodeRef {
    SourceNodeRef::function("motion.veac", "twice")
}

fn set_body(source: &str) -> SourceEditOperation {
    SourceEditOperation::SetBody {
        target: function(),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: source.into(),
        },
    }
}

fn sources(source: &str) -> BTreeMap<String, String> {
    BTreeMap::from([("motion.veac".into(), source.into())])
}
