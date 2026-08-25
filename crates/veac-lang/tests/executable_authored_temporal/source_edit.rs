use std::fs;

use tempfile::tempdir;
use veac_ir::TemporalValue;
use veac_lang::program::{apply_executable_source_edit_path, prepare_path};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
    SourceTemporalProperty,
};

use super::support::{evaluate, MAIN};

#[test]
fn temporal_body_is_indexed_and_rebuilt_from_source_truth() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, MAIN).unwrap();
    let prepared = prepare_path(&entry).unwrap();
    let site = BodySite::TemporalAnimation {
        property: SourceTemporalProperty::VisualOpacity,
    };
    let target = SourceNodeRef::temporal(
        "main.veac",
        "demo",
        "main",
        "visual",
        "first",
        SourceTemporalProperty::VisualOpacity,
    );
    let indexed = prepared.source_index().unwrap();
    assert!(indexed
        .body(&target, site)
        .unwrap()
        .source
        .contains("pulse"));

    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_temporal_source_edit").unwrap(),
        prepared.source_revision().unwrap(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target,
        site,
        body: BodySource {
            source: "{ clip_time / 2s }".to_owned(),
        },
    });
    let before = fs::read_to_string(&entry).unwrap();
    let preview = apply_executable_source_edit_path(&entry, &batch).unwrap();
    assert_eq!(fs::read_to_string(&entry).unwrap(), before);
    assert_ne!(preview.previous_revision, preview.new_revision);
    assert!(preview.source().unwrap().contains("{ clip_time / 2s }"));
    assert_eq!(
        evaluate(
            preview.built.envelope(),
            TemporalValue::Time {
                value: veac_ir::RationalTime::new(1, 1).unwrap()
            }
        ),
        TemporalValue::Scalar { value: 0.5 }
    );
}
