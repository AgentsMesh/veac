use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use veac_artifact::{
    ContentDigest, DeliveryPackageInventory, DeliveryPackageMember, DeliveryPackageNodeType,
};

use super::super::{directory::Directory, StagedFile, StagedOutput, StagedPackage, StagedTask};
use super::{Publication, PublicationFailure};
use crate::{RuntimeError, RuntimeErrorKind};

#[test]
fn invalid_package_and_destination_collision_are_rejected() {
    let root = tempfile::tempdir().unwrap();
    let mut publication = Publication::new(root.path()).unwrap();
    let package = package_task(root.path());
    let error = publication.adopt(package).unwrap_err();
    assert!(error.message.contains("cannot adopt invalid package"));

    let mut publication = Publication::new(root.path()).unwrap();
    publication.descriptor.create_child("entry-000000").unwrap();
    let file = file_task(root.path(), Path::new("payload"), true);
    let error = publication.adopt(file).unwrap_err();
    assert!(error.message.contains("rename transaction path"));
}

#[test]
fn expired_publication_is_classified_before_the_commit_point() {
    let root = tempfile::tempdir().unwrap();
    let parent = std::fs::canonicalize(root.path()).unwrap();
    let locks = crate::executor::locking::acquire_until(
        &[parent],
        Instant::now() + Duration::from_secs(10),
    )
    .unwrap();
    let failure = Publication::new(root.path())
        .unwrap()
        .commit(&locks, Instant::now())
        .unwrap_err();
    assert!(!failure.committed());
    assert_eq!(failure.into_error().kind, RuntimeErrorKind::ResourceLimit);

    let failure = PublicationFailure::before_commit(RuntimeError::new("fixture"));
    assert!(!failure.committed());
    assert_eq!(failure.into_error().message, "fixture");
}

#[cfg(unix)]
#[test]
fn non_utf8_staged_source_name_is_rejected() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let root = tempfile::tempdir().unwrap();
    let source = PathBuf::from(OsString::from_vec(vec![0xff]));
    let task = file_task(root.path(), &source, false);
    let error = Publication::new(root.path())
        .unwrap()
        .adopt(task)
        .unwrap_err();
    assert!(error.message.contains("valid UTF-8"));
}

fn file_task(parent: &Path, source_name: &Path, write_source: bool) -> StagedTask {
    let directory = tempfile::tempdir_in(parent).unwrap();
    let descriptor = Directory::open(directory.path()).unwrap();
    if write_source {
        std::fs::write(directory.path().join(source_name), b"content").unwrap();
    }
    StagedTask {
        outputs: vec![StagedOutput::File(StagedFile {
            source: directory.path().join(source_name),
            target: parent.join("output.mp4"),
            allow_empty: false,
        })],
        directory,
        descriptor,
        stale: Vec::new(),
    }
}

fn package_task(parent: &Path) -> StagedTask {
    let directory = tempfile::tempdir_in(parent).unwrap();
    let descriptor = Directory::open(directory.path()).unwrap();
    std::fs::create_dir(directory.path().join("package")).unwrap();
    let member = DeliveryPackageMember {
        path: "master.m3u8".into(),
        node_type: DeliveryPackageNodeType::RegularFile,
        size_bytes: 1,
        content: Some(ContentDigest::sha256(b"x")),
    };
    let mut inventory = DeliveryPackageInventory::new("master.m3u8", vec![member]).unwrap();
    inventory.tree = ContentDigest::sha256(b"tampered");
    StagedTask {
        outputs: vec![StagedOutput::Package(StagedPackage {
            source: directory.path().join("package"),
            target: parent.join("stream"),
            entrypoint: "master.m3u8".into(),
            inventory,
        })],
        directory,
        descriptor,
        stale: Vec::new(),
    }
}
