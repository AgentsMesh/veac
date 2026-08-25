use std::collections::BTreeMap;

use crate::source_edit::{
    apply_resolved_text_replacements, resolve_source_edit_text, BodySite, BodySource,
    DeclarationSite, DeclarationSource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
    SourceTemporalProperty,
};

const FUNCTION_SOURCE: &str = r#"module {
  fn animated(card: Item) -> Item {
    let faded = animate visual-opacity on clip(card) {
      progress
    };
    animate visual-rotation on clip(faded) {
      progress * 90deg
    }
  }
}"#;

const METHOD_SOURCE: &str = r#"module {
  struct Motion {}
  impl Motion @motion {
    fn attach(self, card: Item) -> Item {
      animate visual-opacity on clip(card) { progress }
    }
  }
}"#;

#[test]
fn indexes_component_animations_in_stable_source_order() {
    let index =
        super::SourceIndex::build_snapshot(&sources("motion.veac", FUNCTION_SOURCE)).unwrap();
    let target = SourceNodeRef::function("motion.veac", "animated");
    let first = body_site(0, SourceTemporalProperty::VisualOpacity);
    let second = body_site(1, SourceTemporalProperty::VisualRotation);
    assert_eq!(
        index.body(&target, first).unwrap().source,
        "{\n      progress\n    }"
    );
    assert_eq!(
        index.body(&target, second).unwrap().source,
        "{\n      progress * 90deg\n    }"
    );
    for ordinal in 0..2 {
        let value = index
            .declaration(&target, DeclarationSite::ComponentAnimation { ordinal })
            .unwrap();
        assert_eq!(
            &FUNCTION_SOURCE[value.range.start..value.range.end],
            value.source
        );
    }
}

#[test]
fn function_animation_body_edit_changes_only_the_authored_block() {
    let target = SourceNodeRef::function("motion.veac", "animated");
    let site = body_site(0, SourceTemporalProperty::VisualOpacity);
    let operation = SourceEditOperation::SetBody {
        target: target.clone(),
        site,
        body: BodySource {
            source: "{ clamp(progress * 2.0, 0.0, 1.0) }".into(),
        },
    };
    let edited = apply(FUNCTION_SOURCE, target, operation);
    assert!(edited.contains("clip(card) { clamp(progress * 2.0, 0.0, 1.0) }"));
    let rebuilt = super::SourceIndex::build_snapshot(&sources("motion.veac", &edited)).unwrap();
    assert_eq!(
        rebuilt
            .body(&SourceNodeRef::function("motion.veac", "animated"), site)
            .unwrap()
            .source,
        "{ clamp(progress * 2.0, 0.0, 1.0) }"
    );
}

#[test]
fn method_animation_declaration_edit_round_trips_from_source() {
    let target = SourceNodeRef::method("motion.veac", "Motion", "attach");
    let site = DeclarationSite::ComponentAnimation { ordinal: 0 };
    let replacement = "animate visual-opacity on clip(card) { 1.0 - progress }";
    let operation = SourceEditOperation::SetDeclaration {
        target: target.clone(),
        site,
        declaration: DeclarationSource {
            source: replacement.into(),
        },
    };
    let edited = apply(METHOD_SOURCE, target.clone(), operation);
    let rebuilt = super::SourceIndex::build_snapshot(&sources("motion.veac", &edited)).unwrap();
    assert_eq!(
        rebuilt.declaration(&target, site).unwrap().source,
        replacement
    );
    assert_eq!(
        rebuilt
            .body(&target, body_site(0, SourceTemporalProperty::VisualOpacity))
            .unwrap()
            .source,
        "{ 1.0 - progress }"
    );
}

fn apply(source: &str, target: SourceNodeRef, operation: SourceEditOperation) -> String {
    let index = super::SourceIndex::build_snapshot(&sources("motion.veac", source)).unwrap();
    let revision = super::super::test_revision(&index);
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_component_animation_edit").unwrap(),
        revision.clone(),
    );
    batch.operations.push(operation.clone());
    crate::source_edit::validate_source_edit_batch(&batch, &revision, &index).unwrap();
    let range = match &operation {
        SourceEditOperation::SetBody { site, .. } => index.body(&target, *site).unwrap().range,
        SourceEditOperation::SetDeclaration { site, .. } => {
            index.declaration(&target, *site).unwrap().range
        }
        _ => unreachable!(),
    };
    let edit = resolve_source_edit_text(0, &operation, range).unwrap();
    apply_resolved_text_replacements("motion.veac", source, &[edit]).unwrap()
}

fn body_site(ordinal: u32, property: SourceTemporalProperty) -> BodySite {
    BodySite::ComponentAnimation { ordinal, property }
}

fn sources(path: &str, source: &str) -> BTreeMap<String, String> {
    BTreeMap::from([(path.into(), source.into())])
}
