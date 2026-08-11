use super::*;
use veac_build::{ProjectActionKind, ProjectBackend};

#[test]
fn action_identities_bind_only_their_exact_required_tools() {
    let ffmpeg = ContentDigest::sha256(b"ffmpeg");
    let ffprobe = ContentDigest::sha256(b"ffprobe");
    let render = render(ffmpeg.clone());
    let derivation = derivation(ffmpeg.clone(), ffprobe.clone());
    let evidence = evidence(ffmpeg, ffprobe);

    let render = render.digest_for(ProjectActionKind::VeacRender).unwrap();
    let derivation = derivation
        .digest_for(ProjectActionKind::MediaDerivation)
        .unwrap();
    let evidence = evidence.digest_for(ProjectActionKind::Evidence).unwrap();
    assert_ne!(render, derivation);
    assert_ne!(derivation, evidence);
    assert_ne!(render, evidence);
}

#[test]
fn compile_time_backend_inventories_are_exact_digests() {
    for value in [PROJECT_BACKEND, PROJECT_BUILD, EVIDENCE_BACKEND] {
        exact(value).validate().unwrap();
    }
    assert_ne!(PROJECT_BACKEND, PROJECT_BUILD);
    assert_ne!(PROJECT_BUILD, EVIDENCE_BACKEND);
}

#[cfg(unix)]
#[test]
fn tool_snapshot_failures_block_outer_cache_lookup() {
    use std::os::unix::fs::PermissionsExt;

    use veac_runtime::asset::SystemFfprobe;
    use veac_runtime::executor::SystemFfmpeg;

    let temp = tempfile::tempdir().unwrap();
    let missing = temp.path().join("missing");
    let unavailable = backend(SystemFfmpeg::new(&missing), SystemFfprobe::new(&missing));
    assert!(unavailable
        .implementation_identity(ProjectActionKind::VeacRender)
        .unwrap_err()
        .message()
        .contains("FFmpeg"));

    let ffmpeg = temp.path().join("ffmpeg");
    std::fs::write(&ffmpeg, b"#!/bin/sh\nprintf 'ffmpeg version test\\n'\n").unwrap();
    std::fs::set_permissions(&ffmpeg, std::fs::Permissions::from_mode(0o700)).unwrap();
    let unavailable = backend(SystemFfmpeg::new(ffmpeg), SystemFfprobe::new(missing));
    assert!(unavailable
        .implementation_identity(ProjectActionKind::MediaDerivation)
        .unwrap_err()
        .message()
        .contains("ffprobe"));
}

#[cfg(unix)]
fn backend(
    ffmpeg: veac_runtime::executor::SystemFfmpeg,
    ffprobe: veac_runtime::asset::SystemFfprobe,
) -> super::super::CliProjectBackend {
    super::super::CliProjectBackend::with_tools(
        std::path::PathBuf::from("source"),
        std::path::PathBuf::from("material"),
        veac_artifact::ArtifactStore::new("artifacts"),
        ContentDigest::sha256(b"manifest"),
        ffmpeg,
        ffprobe,
    )
}
