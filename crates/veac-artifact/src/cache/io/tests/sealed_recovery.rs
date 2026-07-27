use crate::{artifact_key, test_support, ArtifactErrorKind, ArtifactStore};

use super::super::{authority::target_parent, directory::BoundDirectory, recovery};

#[test]
fn recovery_helper_refuses_a_sealed_directory() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = test_support::descriptor();
    let key = artifact_key(&descriptor).unwrap();
    store.put(&descriptor, b"payload").unwrap();
    let directory = store
        .root()
        .join("sha256")
        .join(&key.value[..2])
        .join(&key.value[2..]);
    let (parent, name) = target_parent(store.root(), &directory, false)
        .unwrap()
        .unwrap();
    let bound = BoundDirectory::open(parent, name).unwrap().unwrap();

    assert_eq!(
        recovery::remove_incomplete(bound).unwrap_err().kind,
        ArtifactErrorKind::CorruptCache
    );
}
