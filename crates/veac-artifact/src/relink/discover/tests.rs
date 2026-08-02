#![cfg(unix)]

use std::os::unix::fs::symlink;
use std::time::Duration;

use super::*;
use crate::VerifiedSourceCopy;

#[test]
fn descriptor_walk_rejects_directory_replacement_after_enumeration() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("root");
    let nested = root.join("nested");
    let parked = root.join("parked");
    let external = temp.path().join("external");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::create_dir(&external).unwrap();
    std::fs::write(external.join("escaped"), b"escaped").unwrap();
    let bound_nested = std::fs::canonicalize(&nested).unwrap();

    let error = discover_with_hook(
        &[root],
        RelinkDiscoveryLimits::default(),
        &mut |point, path| {
            if point == DiscoveryHookPoint::EntryEnumerated && path == bound_nested {
                std::fs::rename(&nested, &parked).unwrap();
                symlink(&external, &nested).unwrap();
            }
        },
    )
    .unwrap_err();
    assert!(matches!(
        error.kind,
        ArtifactErrorKind::UnsafePath | ArtifactErrorKind::IdentityMismatch
    ));
}

#[test]
fn descriptor_walk_rejects_file_replacement_after_hashing() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("root");
    let media = root.join("media");
    let parked = root.join("parked");
    let external = temp.path().join("external");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(&media, b"media").unwrap();
    std::fs::write(&external, b"external").unwrap();
    let bound_media = std::fs::canonicalize(&media).unwrap();

    let error = discover_with_hook(
        &[root],
        RelinkDiscoveryLimits::default(),
        &mut |point, path| {
            if point == DiscoveryHookPoint::FileHashed && path == bound_media {
                std::fs::rename(&media, &parked).unwrap();
                symlink(&external, &media).unwrap();
            }
        },
    )
    .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
}

#[test]
fn descriptor_walk_rejects_root_replacement_after_canonical_binding() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("root");
    let parked = temp.path().join("parked");
    let external = temp.path().join("external");
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(&external).unwrap();
    std::fs::write(root.join("media"), b"media").unwrap();
    let bound_root = std::fs::canonicalize(&root).unwrap();

    let error = discover_with_hook(
        std::slice::from_ref(&root),
        RelinkDiscoveryLimits::default(),
        &mut |point, path| {
            if point == DiscoveryHookPoint::RootVisited && path == bound_root {
                std::fs::rename(&root, &parked).unwrap();
                symlink(&external, &root).unwrap();
            }
        },
    )
    .unwrap_err();
    assert!(matches!(
        error.kind,
        ArtifactErrorKind::UnsafePath | ArtifactErrorKind::IdentityMismatch
    ));
}

#[test]
fn one_wall_deadline_covers_enumeration_hooks_and_hashing() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("media"), b"media").unwrap();
    let limits = RelinkDiscoveryLimits {
        max_wall_time: Duration::from_millis(5),
        ..RelinkDiscoveryLimits::default()
    };
    let error = discover_with_hook(&[temp.path().to_owned()], limits, &mut |point, _| {
        if point == DiscoveryHookPoint::EntryEnumerated {
            std::thread::sleep(Duration::from_millis(15));
        }
    })
    .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
}

#[test]
fn aggregate_and_search_root_budgets_are_enforced_before_walking() {
    let limits = RelinkDiscoveryLimits {
        max_entries: 1,
        max_file_bytes: 3,
        max_total_bytes: 3,
        ..RelinkDiscoveryLimits::default()
    };
    let mut budget = DiscoveryBudget::new(limits, Instant::now()).unwrap();
    let verified = |bytes: &[u8]| VerifiedSourceCopy {
        identity: crate::test_support::media_identity(bytes),
        size_bytes: bytes.len() as u64,
    };
    budget.candidate("first".into(), verified(b"ab")).unwrap();
    assert_eq!(
        budget
            .candidate("second".into(), verified(b"cd"))
            .unwrap_err()
            .kind,
        ArtifactErrorKind::ResourceLimit
    );

    let roots = ["first".into(), "second".into()];
    assert_eq!(
        discover_with_hook(&roots, limits, &mut |_, _| {})
            .unwrap_err()
            .kind,
        ArtifactErrorKind::ResourceLimit
    );
}
