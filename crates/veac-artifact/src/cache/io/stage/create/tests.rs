use super::*;
use crate::ArtifactErrorKind;

#[test]
fn setup_cleanup_removes_every_bound_entry_and_preserves_the_primary_error() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    let target = root.join("artifact");
    let (parent, name) = super::super::super::authority::target_parent(&root, &target, true)
        .unwrap()
        .unwrap();
    let StageOpen::Ready(stage) = begin(parent, name, || true).unwrap() else {
        panic!("new cache target must start a transaction");
    };
    let primary = ArtifactError::new(ArtifactErrorKind::Io, "primary");
    let error = cleanup_after(
        &stage.directory,
        &[&stage.descriptor, &stage.payload, &stage.record],
        primary,
    );

    assert_eq!(error.kind, ArtifactErrorKind::Io);
    assert!(!target.exists());
}

#[test]
fn every_partial_entry_creation_failure_runs_bound_cleanup() {
    for blocker in [DESCRIPTOR, PAYLOAD, RECORD] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("store");
        let target = root.join("artifact");
        let (parent, name) = super::super::super::authority::target_parent(&root, &target, true)
            .unwrap()
            .unwrap();
        let lock = DirectoryLock::exclusive_while(parent.current(), || true).unwrap();
        let directory = BoundDirectory::create(parent, name).unwrap();
        std::fs::write(directory.path().join(blocker), b"blocker").unwrap();

        let error = create_entries(directory, lock).err().unwrap();
        assert!(matches!(
            error.kind,
            ArtifactErrorKind::Io | ArtifactErrorKind::UnsafePath
        ));
        assert_eq!(std::fs::read(target.join(blocker)).unwrap(), b"blocker");
    }
}
