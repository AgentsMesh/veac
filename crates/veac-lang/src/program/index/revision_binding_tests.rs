use std::collections::BTreeMap;

use crate::program::prepare_source;
use crate::source_edit::SourceEditError;

use super::inventory_tests::SOURCE;

#[test]
fn inventory_rejects_arbitrary_valid_complete_digest() {
    let prepared = prepare_source(SOURCE).unwrap();
    let index = prepared.source_index().unwrap();
    let mut forged = prepared.source_revision().unwrap();
    forged.complete_source_graph_sha256 = "0".repeat(64);
    assert_ne!(
        forged.complete_source_graph_sha256,
        prepared.source_graph().complete_revision().sha256()
    );
    assert!(matches!(
        index.inventory(&forged),
        Err(SourceEditError::StaleRevision { .. })
    ));
}

#[test]
fn inventory_rejects_mismatched_authored_and_malformed_revisions() {
    let prepared = prepare_source(SOURCE).unwrap();
    let index = prepared.source_index().unwrap();
    let mut stale = prepared.source_revision().unwrap();
    stale.authored_source_graph_sha256 = "0".repeat(64);
    assert!(matches!(
        index.inventory(&stale),
        Err(SourceEditError::StaleRevision { .. })
    ));

    let mut malformed = prepared.source_revision().unwrap();
    malformed.complete_source_graph_sha256 = "invalid".to_owned();
    assert_eq!(
        index.inventory(&malformed).unwrap_err(),
        SourceEditError::InvalidDigest("invalid".to_owned())
    );
}

#[test]
fn snapshot_only_index_cannot_publish_inventory() {
    let prepared = prepare_source(SOURCE).unwrap();
    let index = super::SourceIndex::build_snapshot(&BTreeMap::from([(
        "main.veac".to_owned(),
        SOURCE.to_owned(),
    )]))
    .unwrap();
    let error = index
        .inventory(&prepared.source_revision().unwrap())
        .unwrap_err();
    assert_eq!(error, SourceEditError::UnboundSourceIndex);
    assert_eq!(
        error.to_string(),
        "source index is not bound to a prepared source graph"
    );
}
