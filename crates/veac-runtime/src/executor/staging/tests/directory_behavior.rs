#![cfg(unix)]

use std::ffi::OsString;
use std::os::unix::fs::symlink;

use super::*;

#[test]
fn descriptor_directory_round_trips_bounded_regular_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = Directory::open(temp.path()).unwrap();
    let transaction = root.create_child("transaction").unwrap();
    let second = root.child("transaction").unwrap();
    let identity = transaction.write_all_sync("payload", b"content").unwrap();

    assert_eq!(
        transaction.state("payload").unwrap(),
        EntryState::Regular(identity)
    );
    assert!(!transaction.missing("payload").unwrap());
    assert!(transaction.missing("absent").unwrap());
    assert_eq!(
        transaction.read_bounded("payload", 7).unwrap().0,
        b"content"
    );
    assert!(transaction.read_bounded("payload", 6).is_err());
    assert!(transaction.read_bounded("absent", 7).is_err());
    transaction.sync_bound("payload", identity).unwrap();
    transaction.sync().unwrap();

    assert!(second.write_all_sync("payload", b"again").is_err());
    transaction.remove_bound("payload", identity).unwrap();
    transaction.remove_if_regular("payload").unwrap();
    root.remove_child("transaction", &transaction).unwrap();
}

#[test]
fn rename_and_listing_stay_bound_to_descriptor_identities() {
    let temp = tempfile::tempdir().unwrap();
    let root = Directory::open(temp.path()).unwrap();
    let source = root.create_child("source").unwrap();
    let target = root.create_child("target").unwrap();
    let first = source.write_all_sync("b", b"bb").unwrap();
    source.write_all_sync("a", b"a").unwrap();
    assert_eq!(
        source.entries(2).unwrap(),
        vec![OsString::from("a"), OsString::from("b")]
    );
    assert_eq!(
        source.entries(1).unwrap_err().kind,
        crate::RuntimeErrorKind::ResourceLimit
    );

    let EntryIdentity::Regular {
        device,
        inode,
        size_bytes,
    } = first
    else {
        panic!("file identity is regular");
    };
    let mismatch = EntryIdentity::Regular {
        device,
        inode: inode.wrapping_add(1),
        size_bytes,
    };
    let failure = source
        .rename_bound_to("b", mismatch, &target, "moved")
        .unwrap_err();
    assert!(!failure.crossed_commit);
    source
        .rename_bound_to("b", first, &target, "moved")
        .unwrap();
    assert_eq!(target.read_bounded("moved", 2).unwrap().0, b"bb");

    source.remove_if_regular("a").unwrap();
    target.remove_if_regular("moved").unwrap();
    assert!(root.remove_child("source", &target).is_err());
    root.remove_child("source", &source).unwrap();
    root.remove_child("target", &target).unwrap();
}

#[test]
fn descriptor_operations_reject_links_and_replaced_identities() {
    let temp = tempfile::tempdir().unwrap();
    let root = Directory::open(temp.path()).unwrap();
    let original_path = temp.path().join("original");
    std::fs::write(&original_path, b"one").unwrap();
    let original_guard = std::fs::File::open(&original_path).unwrap();
    let EntryState::Regular(original) = root.state("original").unwrap() else {
        panic!("regular fixture");
    };
    std::fs::remove_file(&original_path).unwrap();
    std::fs::write(&original_path, b"two").unwrap();
    assert!(root.require("original", original).is_err());
    assert!(root.sync_bound("original", original).is_err());
    drop(original_guard);

    symlink("original", temp.path().join("symbolic")).unwrap();
    assert!(root
        .state("symbolic")
        .unwrap_err()
        .message
        .contains("regular file"));
    std::fs::hard_link(temp.path().join("original"), temp.path().join("hard")).unwrap();
    assert!(root
        .state("hard")
        .unwrap_err()
        .message
        .contains("exclusively linked"));
}
