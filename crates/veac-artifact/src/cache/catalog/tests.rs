use super::*;
use crate::{test_support, ArtifactParameters};

#[test]
fn catalog_enforces_aggregate_metadata_and_payload_budgets_before_hashing() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let first = test_support::descriptor();
    let mut second = first.clone();
    let ArtifactParameters::ProxyVideo(parameters) = &mut second.parameters else {
        unreachable!()
    };
    parameters.crf = 25;
    store.put(&first, b"first-a").unwrap();
    store.put(&second, b"second!").unwrap();
    let query = ArtifactCatalogQuery::new(first.kind(), first.dependencies.clone()).unwrap();

    assert_eq!(
        catalog_with_budgets(&store, &query, 1, 14)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::ResourceLimit
    );
    assert_eq!(
        catalog_with_budgets(&store, &query, crate::MAX_IN_MEMORY_ARTIFACT_BYTES, 7)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::ResourceLimit
    );
    assert_eq!(store.catalog(&query).unwrap().len(), 2);
}
