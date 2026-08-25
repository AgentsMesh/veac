use std::collections::BTreeMap;

use crate::source_edit::{
    apply_resolved_text_replacements, resolve_source_edit_text, BodySite, BodySource,
    SourceEditBatch, SourceEditOperation, SourceNodeRef, SourcePrecondition, SourceSnapshot,
};

const SOURCE: &str = r#"module {
  export struct Timing { duration: time, }
  impl Timing @timing {
    export fn padded(self, extra: time) -> time {
      self.duration + extra
    }
  }
}"#;

#[test]
fn indexes_method_nodes_bodies_and_exact_source_ranges() {
    let index = super::SourceIndex::build_snapshot(&sources(SOURCE)).unwrap();
    let body = index.body(&target(), BodySite::MethodBody).unwrap();
    assert!(index.node_exists(&target()));
    assert_eq!(body.source, "{\n      self.duration + extra\n    }");
    assert_eq!(&SOURCE[body.range.start..body.range.end], body.source);
}

#[test]
fn method_body_edit_round_trips_through_a_rebuilt_index() {
    let index = super::SourceIndex::build_snapshot(&sources(SOURCE)).unwrap();
    let revision = super::super::test_revision(&index);
    let operation = set_body("{ self.duration + extra + 250ms }");
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_method_body_index").unwrap(),
        revision.clone(),
    );
    batch.preconditions.push(SourcePrecondition::BodyEquals {
        target: target(),
        site: BodySite::MethodBody,
        body: BodySource {
            source: "{\n      self.duration + extra\n    }".into(),
        },
    });
    batch.operations.push(operation.clone());
    crate::source_edit::validate_source_edit_batch(&batch, &revision, &index).unwrap();
    let range = index.body(&target(), BodySite::MethodBody).unwrap().range;
    let replacement = resolve_source_edit_text(0, &operation, range).unwrap();
    let edited = apply_resolved_text_replacements("brand.veac", SOURCE, &[replacement]).unwrap();
    let rebuilt = super::SourceIndex::build_snapshot(&sources(&edited)).unwrap();
    assert_eq!(
        rebuilt
            .body(&target(), BodySite::MethodBody)
            .unwrap()
            .source,
        "{ self.duration + extra + 250ms }"
    );
    assert_ne!(rebuilt.revision(), index.revision());
}

#[test]
fn moving_a_method_between_impl_identities_preserves_its_target() {
    let first = super::SourceIndex::build_snapshot(&sources(SOURCE)).unwrap();
    let moved = SOURCE.replace("@timing", "@presentation");
    let second = super::SourceIndex::build_snapshot(&sources(&moved)).unwrap();
    assert!(first.node_exists(&target()));
    assert!(second.node_exists(&target()));
    assert_eq!(
        first.body(&target(), BodySite::MethodBody).unwrap().source,
        second.body(&target(), BodySite::MethodBody).unwrap().source
    );
}

fn target() -> SourceNodeRef {
    SourceNodeRef::method("brand.veac", "Timing", "padded")
}

fn set_body(source: &str) -> SourceEditOperation {
    SourceEditOperation::SetBody {
        target: target(),
        site: BodySite::MethodBody,
        body: BodySource {
            source: source.into(),
        },
    }
}

fn sources(source: &str) -> BTreeMap<String, String> {
    BTreeMap::from([("brand.veac".into(), source.into())])
}
