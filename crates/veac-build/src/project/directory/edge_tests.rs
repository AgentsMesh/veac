use std::io::Write;

use sha2::Digest;
use veac_artifact::ArtifactStore;

use super::{pack, publish, relative_text, unpack, ENTRY_END, ENTRY_FILE, MAGIC, MAX_PATH_BYTES};
use crate::{BuildErrorKind, CancellationToken, ContentDigest, ExecutionErrorKind};

#[test]
fn unpack_rejects_each_closed_archive_format_error() {
    let mut trailing = MAGIC.to_vec();
    trailing.extend_from_slice(&[ENTRY_END, 1]);
    let mut unknown = MAGIC.to_vec();
    unknown.push(99);
    let mut empty_path = MAGIC.to_vec();
    empty_path.push(ENTRY_FILE);
    empty_path.extend_from_slice(&0_u32.to_be_bytes());
    let mut long_path = MAGIC.to_vec();
    long_path.push(ENTRY_FILE);
    long_path.extend_from_slice(&((MAX_PATH_BYTES + 1) as u32).to_be_bytes());
    let mut invalid_utf8 = MAGIC.to_vec();
    invalid_utf8.push(ENTRY_FILE);
    invalid_utf8.extend_from_slice(&1_u32.to_be_bytes());
    invalid_utf8.push(0xff);
    let mut bad_digest = archive("file", b"payload");
    let payload = bad_digest.len() - 2;
    bad_digest[payload] ^= 1;

    for bytes in [
        b"INVALID!".to_vec(),
        trailing,
        unknown,
        empty_path,
        long_path,
        invalid_utf8,
        bad_digest,
        MAGIC.to_vec(),
    ] {
        reject_archive(&bytes);
    }
}

#[test]
fn pack_maps_missing_archives_and_rejects_nonregular_entries() {
    let temp = tempfile::tempdir().unwrap();
    let error = pack(
        &temp.path().join("missing"),
        &temp.path().join("archive"),
        &CancellationToken::new(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), ExecutionErrorKind::Failed);

    let source = temp.path().join("source");
    std::fs::create_dir(&source).unwrap();
    let archive = temp.path().join("existing");
    std::fs::write(&archive, b"x").unwrap();
    assert!(pack(&source, &archive, &CancellationToken::new()).is_err());

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(temp.path(), source.join("link")).unwrap();
        let error = pack(
            &source,
            &temp.path().join("symlink-archive"),
            &CancellationToken::new(),
        )
        .unwrap_err();
        assert!(error.message().contains("non-regular"));
    }
}

#[test]
fn directory_paths_and_publication_reject_unsafe_authorities() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("root");
    std::fs::create_dir(&root).unwrap();
    let root = std::fs::canonicalize(root).unwrap();
    assert!(relative_text(&root, temp.path()).is_err());
    assert!(relative_text(&root, &root).is_err());
    assert!(relative_text(&root, &root.join("nested/../file")).is_err());

    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        let invalid = root.join(std::ffi::OsString::from_vec(vec![0xff]));
        assert!(relative_text(&root, &invalid).is_err());
    }

    let artifact = ContentDigest::sha256(b"missing");
    let store = ArtifactStore::new(temp.path().join("artifacts"));
    let error = publish(
        &store,
        &artifact,
        &root,
        std::path::Path::new("bundle"),
        &CancellationToken::new(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), BuildErrorKind::Cache, "{error}");

    let broken = temp.path().join("store-file");
    std::fs::write(&broken, b"x").unwrap();
    let error = publish(
        &ArtifactStore::new(broken),
        &artifact,
        &root,
        std::path::Path::new("bundle"),
        &CancellationToken::new(),
    )
    .unwrap_err();
    assert!(
        error.message().contains("artifact delivery failure"),
        "{error}"
    );
}

fn reject_archive(bytes: &[u8]) {
    let temp = tempfile::tempdir().unwrap();
    let archive = temp.path().join("archive");
    let destination = temp.path().join("destination");
    std::fs::write(&archive, bytes).unwrap();
    std::fs::create_dir(&destination).unwrap();
    assert_eq!(
        unpack(&archive, &destination, &CancellationToken::new())
            .unwrap_err()
            .kind(),
        BuildErrorKind::Cache
    );
}

fn archive(path: &str, bytes: &[u8]) -> Vec<u8> {
    let mut output = MAGIC.to_vec();
    output.push(ENTRY_FILE);
    output.extend_from_slice(&(path.len() as u32).to_be_bytes());
    output.extend_from_slice(path.as_bytes());
    output.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    output.extend_from_slice(&sha2::Sha256::digest(bytes));
    output.write_all(bytes).unwrap();
    output.push(ENTRY_END);
    output
}
