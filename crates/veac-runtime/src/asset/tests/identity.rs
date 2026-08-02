use std::error::Error;
use std::io;

use tempfile::tempdir;
use veac_ir::ProbedStreamType;

use super::*;

#[test]
fn hashes_local_files_with_sha256() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("fixture.bin");
    std::fs::write(&path, b"abc").unwrap();
    let identity = sha256_identity(&path).unwrap();
    assert_eq!(identity.algorithm, HashAlgorithm::Sha256);
    assert_eq!(
        identity.digest,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );

    let large = temp.path().join("large.bin");
    std::fs::write(&large, vec![7_u8; 70_000]).unwrap();
    assert_eq!(sha256_identity(&large).unwrap().digest.len(), 64);
}

#[test]
fn local_probe_reports_file_spawn_and_process_failures() {
    let temp = tempdir().unwrap();
    let missing = temp.path().join("missing.mp4");
    let missing_error = probe(&missing).unwrap_err();
    assert!(matches!(&missing_error, ProbeError::FileNotFound { .. }));
    assert!(missing_error.to_string().contains("missing.mp4"));
    assert!(matches!(
        probe_with_intent(&missing, auto_stream_intent()),
        Err(ProbeError::FileNotFound { .. })
    ));
    let hash_error = sha256_identity(&missing).unwrap_err();
    assert!(matches!(
        &hash_error,
        ProbeError::Io { path, .. } if path == &missing
    ));
    assert!(hash_error.source().is_some());

    let invalid = temp.path().join("invalid.txt");
    std::fs::write(&invalid, "not media").unwrap();
    assert!(matches!(
        SystemFfprobe::new("veac-no-such-ffprobe")
            .probe_with_intent(&invalid, auto_stream_intent()),
        Err(ProbeError::ProcessSpawn { .. })
    ));
    let false_result =
        SystemFfprobe::new("false").probe_with_intent(&invalid, auto_stream_intent());
    assert!(
        matches!(
            &false_result,
            Err(ProbeError::VersionFailed {
                status: None | Some(_),
                ..
            })
        ),
        "{false_result:?}"
    );
    assert!(matches!(
        SystemFfprobe::new("true").probe_with_intent(&invalid, auto_stream_intent()),
        Err(ProbeError::InvalidField {
            field: "engine.version",
            ..
        })
    ));
    assert!(matches!(
        probe(&invalid),
        Err(ProbeError::ProcessFailed { .. })
    ));
}

#[test]
fn structured_errors_expose_context_and_sources() {
    let path = std::path::PathBuf::from("media.mp4");
    let io_error = ProbeError::Io {
        operation: "hash",
        path: path.clone(),
        source: io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
    };
    assert!(io_error.to_string().contains("failed to hash media.mp4"));
    assert!(io_error.source().is_some());

    let spawn = ProbeError::ProcessSpawn {
        binary: "custom-probe".into(),
        source: io::Error::new(io::ErrorKind::NotFound, "missing"),
    };
    assert!(spawn.to_string().contains("custom-probe"));
    assert!(spawn.source().is_some());

    for (status, expected) in [(Some(9), "status 9"), (None, "status signal")] {
        let failed = ProbeError::ProcessFailed {
            path: path.clone(),
            status,
            stderr: "bad stream".into(),
        };
        let rendered = failed.to_string();
        assert!(rendered.contains(expected) && rendered.contains("bad stream"));
        assert!(failed.source().is_none());
    }

    let selection = ProbeError::StreamSelection {
        media_type: ProbedStreamType::Video,
        global_index: 7,
    };
    assert!(selection.to_string().contains("stream 7"));

    let version = ProbeError::VersionFailed {
        binary: "ffprobe".into(),
        status: None,
        stderr: "terminated".into(),
    };
    assert!(version.to_string().contains("status signal"));

    let changed = ProbeError::IdentityChanged { path };
    assert!(changed
        .to_string()
        .contains("media changed while being probed"));
}

#[test]
fn hashing_a_directory_returns_a_typed_io_error() {
    let temp = tempdir().unwrap();
    let error = sha256_identity(temp.path()).unwrap_err();
    assert!(matches!(error, ProbeError::Io { .. }));
}
