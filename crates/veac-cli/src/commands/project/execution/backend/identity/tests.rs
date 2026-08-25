use super::*;
use veac_build::{ProjectAction, ProjectActionKind, ProjectBackend, ProjectComputation};

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
    let render = action(ProjectActionKind::VeacRender);
    assert!(unavailable
        .implementation_identity(&render)
        .unwrap_err()
        .message()
        .contains("FFmpeg"));

    let ffmpeg = temp.path().join("ffmpeg");
    std::fs::write(&ffmpeg, b"#!/bin/sh\nprintf 'ffmpeg version test\\n'\n").unwrap();
    std::fs::set_permissions(&ffmpeg, std::fs::Permissions::from_mode(0o700)).unwrap();
    let unavailable = backend(SystemFfmpeg::new(ffmpeg), SystemFfprobe::new(missing));
    let derivation = action(ProjectActionKind::MediaDerivation);
    assert!(unavailable
        .implementation_identity(&derivation)
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
        veac_build::ProjectPackageSet::capture(&[]).unwrap(),
        veac_artifact::ArtifactStore::new("artifacts"),
        ContentDigest::sha256(b"manifest"),
        ffmpeg,
        ffprobe,
    )
}

#[cfg(unix)]
fn action(kind: ProjectActionKind) -> ProjectAction {
    let computation = ProjectComputation {
        instance: veac_project::TargetInstanceId::from("instance"),
        target: veac_project::TargetId::from("target"),
        profile: None,
        locale: None,
        matrix: Default::default(),
        inputs: Vec::new(),
        outputs: Vec::new(),
        bound_sources: Vec::new(),
        package_mounts: Vec::new(),
    };
    match kind {
        ProjectActionKind::MediaDerivation => ProjectAction::MediaDerivation {
            computation,
            operation: veac_project::MediaDerivation::Thumbnail {
                source: veac_project::InputId::from("source"),
                source_stream: veac_project::ProjectStreamSelection {
                    global_index: 0,
                    type_index: 0,
                },
                at: veac_project::ProjectRational::new(0, 1),
                width: 1,
                height: 1,
            },
        },
        ProjectActionKind::VeacRender => ProjectAction::VeacRender {
            computation,
            source: snapshot(),
            source_graph: revision(),
        },
        ProjectActionKind::Evidence => ProjectAction::Evidence {
            computation,
            contract: snapshot(),
            source_graph: revision(),
        },
    }
}

#[cfg(unix)]
fn snapshot() -> veac_build::ProjectFileSnapshot {
    veac_build::ProjectFileSnapshot {
        path: "main.veac".to_owned(),
        content: ContentDigest::sha256(b"source"),
        size_bytes: 6,
    }
}

#[cfg(unix)]
fn revision() -> veac_build::ProjectSourceGraphRevision {
    veac_build::ProjectSourceGraphRevision {
        root_module: "main.veac".to_owned(),
        authored_source_graph_sha256: "0".repeat(64),
        complete_source_graph_sha256: "1".repeat(64),
        authored_module_count: 1,
        authored_modules: vec!["main.veac".to_owned()],
    }
}
