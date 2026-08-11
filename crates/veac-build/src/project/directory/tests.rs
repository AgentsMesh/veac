use std::io::Write;

use sha2::Digest;

use super::{pack, unpack, ENTRY_FILE, MAGIC};
use crate::{BuildErrorKind, CancellationToken, ExecutionErrorKind};

#[test]
fn directory_archive_is_deterministic_and_round_trips_nested_content() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    std::fs::create_dir_all(source.join("nested/empty")).unwrap();
    std::fs::write(source.join("z.txt"), b"last").unwrap();
    std::fs::write(source.join("nested/a.txt"), b"first").unwrap();
    let first = temp.path().join("first.dir");
    let second = temp.path().join("second.dir");
    pack(&source, &first, &CancellationToken::new()).unwrap();
    pack(&source, &second, &CancellationToken::new()).unwrap();
    assert_eq!(
        std::fs::read(&first).unwrap(),
        std::fs::read(&second).unwrap()
    );

    let restored = temp.path().join("restored");
    std::fs::create_dir(&restored).unwrap();
    unpack(&first, &restored, &CancellationToken::new()).unwrap();
    assert_eq!(std::fs::read(restored.join("z.txt")).unwrap(), b"last");
    assert_eq!(
        std::fs::read(restored.join("nested/a.txt")).unwrap(),
        b"first"
    );
    assert!(restored.join("nested/empty").is_dir());
}

#[test]
fn pack_rejects_non_directories_and_honors_cancellation() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("file");
    std::fs::write(&file, b"x").unwrap();
    let error = pack(
        &file,
        &temp.path().join("invalid.dir"),
        &CancellationToken::new(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), ExecutionErrorKind::Failed);

    let source = temp.path().join("source");
    std::fs::create_dir(&source).unwrap();
    std::fs::write(source.join("file"), b"x").unwrap();
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let error = pack(&source, &temp.path().join("cancelled.dir"), &cancellation).unwrap_err();
    assert_eq!(error.kind(), ExecutionErrorKind::Cancelled);
}

#[test]
fn unpack_rejects_unsafe_unsorted_corrupt_and_cancelled_archives() {
    let temp = tempfile::tempdir().unwrap();
    for (name, archive) in [
        ("unsafe", archive(&[("../escape", b"x")])),
        ("unsorted", archive(&[("z", b"z"), ("a", b"a")])),
    ] {
        let source = temp.path().join(format!("{name}.dir"));
        std::fs::write(&source, archive).unwrap();
        let destination = temp.path().join(name);
        std::fs::create_dir(&destination).unwrap();
        let error = unpack(&source, &destination, &CancellationToken::new()).unwrap_err();
        assert_eq!(error.kind(), BuildErrorKind::Cache);
    }

    let source = temp.path().join("corrupt.dir");
    let mut bytes = archive(&[("file", b"payload")]);
    *bytes.last_mut().unwrap() ^= 1;
    std::fs::write(&source, bytes).unwrap();
    let destination = temp.path().join("corrupt");
    std::fs::create_dir(&destination).unwrap();
    assert_eq!(
        unpack(&source, &destination, &CancellationToken::new())
            .unwrap_err()
            .kind(),
        BuildErrorKind::Cache
    );

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let destination = temp.path().join("cancelled");
    std::fs::create_dir(&destination).unwrap();
    assert_eq!(
        unpack(&source, &destination, &cancellation)
            .unwrap_err()
            .kind(),
        BuildErrorKind::Cancelled
    );
}

fn archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut output = MAGIC.to_vec();
    for (path, bytes) in entries {
        output.push(ENTRY_FILE);
        output.extend_from_slice(&(path.len() as u32).to_be_bytes());
        output.extend_from_slice(path.as_bytes());
        output.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
        output.extend_from_slice(&sha2::Sha256::digest(bytes));
        output.write_all(bytes).unwrap();
    }
    output.push(super::ENTRY_END);
    output
}
