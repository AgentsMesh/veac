use std::path::{Path, PathBuf};

use veac_artifact::{ArtifactStore, ContentDigest};
use veac_build::{
    ProjectAction, ProjectBackend, ProjectComputation, ProjectFileSnapshot,
    ProjectPackageMountRevision, ProjectSourceGraphRevision,
};
use veac_project::{TargetId, TargetInstanceId};
use veac_runtime::asset::SystemFfprobe;
use veac_runtime::executor::SystemFfmpeg;

use super::CliProjectBackend;

#[test]
fn package_drift_blocks_backend_identity_before_tool_or_cache_lookup() {
    let temp = tempfile::tempdir().unwrap();
    let package_root = temp.path().join("package");
    copy_standard_package(&package_root);
    let packages =
        veac_build::ProjectPackageSet::capture(std::slice::from_ref(&package_root)).unwrap();
    let action = action(packages.revision().to_vec());
    std::fs::write(package_root.join("main.veac"), "changed after planning\n").unwrap();
    let missing = temp.path().join("missing-tool");
    let backend = CliProjectBackend::with_tools(
        temp.path().join("source"),
        temp.path().join("material"),
        packages,
        ArtifactStore::new(temp.path().join("artifacts")),
        ContentDigest::sha256(b"manifest"),
        SystemFfmpeg::new(&missing),
        SystemFfprobe::new(&missing),
    );

    let error = backend.implementation_identity(&action).unwrap_err();
    assert!(error.message().contains("package verification"));
    assert!(!error.message().contains("FFmpeg"));
}

fn action(package_mounts: Vec<ProjectPackageMountRevision>) -> ProjectAction {
    ProjectAction::Evidence {
        computation: ProjectComputation {
            instance: TargetInstanceId::from("instance"),
            target: TargetId::from("target"),
            profile: None,
            locale: None,
            matrix: Default::default(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            bound_sources: Vec::new(),
            package_mounts,
        },
        contract: ProjectFileSnapshot {
            path: "evidence.veac".to_owned(),
            content: ContentDigest::sha256(b"evidence"),
            size_bytes: 8,
        },
        source_graph: ProjectSourceGraphRevision {
            root_module: "evidence.veac".to_owned(),
            authored_source_graph_sha256: "0".repeat(64),
            complete_source_graph_sha256: "1".repeat(64),
            authored_module_count: 1,
            authored_modules: vec!["evidence.veac".to_owned()],
        },
    }
}

fn copy_standard_package(destination: &Path) {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib/veac-components");
    for relative in [
        "main.veac",
        "layout.veac",
        "text.veac",
        "motion.veac",
        "media.veac",
        "audio.veac",
        "delivery.veac",
        "components/card.veac",
        "veac.package.api.json",
        "veac.package.json",
        "veac.package.lock",
    ] {
        let target = destination.join(relative);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(source.join(relative), target).unwrap();
    }
}
