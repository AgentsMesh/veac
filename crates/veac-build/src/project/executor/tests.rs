use super::{identity_error, ProjectNodeExecutor};
use crate::{
    CancellationToken, ContentDigest, ExecuteRequest, NodeCacheKey, NodeExecutor, NodeId, PortName,
    ProjectAction, ProjectBackend, ProjectComputation, ProjectExecutionRequest,
    ProjectFileSnapshot, ProjectSourceGraphRevision, ResolvedInput,
};
use veac_artifact::{ArtifactStore, DigestAlgorithm};
use veac_project::{
    MatrixAssignment, MediaType, OutputId, ProjectOutput, TargetId, TargetInstanceId,
};

struct UnreachableBackend;

impl ProjectBackend for UnreachableBackend {
    fn implementation_identity(
        &self,
        action: &ProjectAction,
    ) -> Result<crate::ProjectBackendIdentity, crate::ProjectBackendError> {
        Ok(test_identity(
            action.kind(),
            ContentDigest::sha256(b"unreachable"),
        ))
    }

    fn execute(
        &self,
        _request: ProjectExecutionRequest<'_>,
        _cancellation: &CancellationToken,
    ) -> Result<Vec<crate::ProducedProjectOutput>, crate::ProjectBackendError> {
        panic!("backend must not run in executor guard tests")
    }
}

fn test_identity(
    action: crate::ProjectActionKind,
    value: ContentDigest,
) -> crate::ProjectBackendIdentity {
    match action {
        crate::ProjectActionKind::VeacRender => crate::ProjectBackendIdentity::VeacRender {
            project_backend: value.clone(),
            build: value.clone(),
            compiler: value.clone(),
            codegen: value.clone(),
            runtime: value.clone(),
            ffmpeg: value,
        },
        _ => unreachable!(),
    }
}

fn action() -> ProjectAction {
    ProjectAction::VeacRender {
        computation: ProjectComputation {
            instance: TargetInstanceId::from("instance"),
            target: TargetId::from("target"),
            profile: None,
            locale: None,
            matrix: MatrixAssignment::new(),
            inputs: Vec::new(),
            outputs: vec![ProjectOutput::Media {
                id: OutputId::from("video"),
                media_type: MediaType::Video,
            }],
            bound_sources: Vec::new(),
            package_mounts: Vec::new(),
        },
        source: ProjectFileSnapshot {
            path: "main.veac".to_owned(),
            content: ContentDigest::sha256(b"source"),
            size_bytes: 6,
        },
        source_graph: ProjectSourceGraphRevision {
            root_module: "main.veac".to_owned(),
            authored_source_graph_sha256: "0".repeat(64),
            complete_source_graph_sha256: "1".repeat(64),
            authored_module_count: 1,
            authored_modules: vec!["main.veac".to_owned()],
        },
    }
}

fn cache_key() -> NodeCacheKey {
    NodeCacheKey::computation("test", 1, ContentDigest::sha256(b"configuration")).unwrap()
}

#[test]
fn executor_rejects_invalid_staging_roots() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("artifacts"));
    let file = temp.path().join("file");
    std::fs::write(&file, b"x").unwrap();
    assert!(
        ProjectNodeExecutor::new(store.clone(), file.join("child"), UnreachableBackend).is_err()
    );

    #[cfg(unix)]
    {
        let actual = temp.path().join("actual");
        let link = temp.path().join("link");
        std::fs::create_dir(&actual).unwrap();
        std::os::unix::fs::symlink(&actual, &link).unwrap();
        let error = ProjectNodeExecutor::new(store, link, UnreachableBackend)
            .err()
            .unwrap();
        assert!(error.message().contains("non-symlink"));
    }
}

#[test]
fn backend_identity_failures_preserve_internal_context() {
    let error = identity_error(crate::ProjectBackendError::failed("fingerprint failed"));
    assert_eq!(error.kind(), crate::BuildErrorKind::Internal);
    assert!(error.message().contains("fingerprint failed"));
}

#[test]
fn executor_stops_before_backend_for_cancellation_missing_inputs_and_staging_loss() {
    let temp = tempfile::tempdir().unwrap();
    let staging = temp.path().join("staging");
    let executor = ProjectNodeExecutor::new(
        ArtifactStore::new(temp.path().join("artifacts")),
        &staging,
        UnreachableBackend,
    )
    .unwrap();
    let node = NodeId::new("node").unwrap();
    let action = action();
    let key = cache_key();
    let token = CancellationToken::new();
    token.cancel();
    let request = ExecuteRequest {
        node_id: &node,
        action: &action,
        inputs: &[],
        cache_key: &key,
    };
    assert_eq!(
        executor.execute(request, &token).unwrap_err().kind(),
        crate::ExecutionErrorKind::Cancelled
    );

    let inputs = [ResolvedInput {
        role: PortName::new("source").unwrap(),
        producer: NodeId::new("producer").unwrap(),
        output: PortName::new("video").unwrap(),
        digest: ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: "0".repeat(64),
        },
    }];
    let request = ExecuteRequest {
        node_id: &node,
        action: &action,
        inputs: &inputs,
        cache_key: &key,
    };
    assert!(executor
        .execute(request, &CancellationToken::new())
        .unwrap_err()
        .message()
        .contains("missing"));

    std::fs::remove_dir_all(&staging).unwrap();
    let request = ExecuteRequest {
        node_id: &node,
        action: &action,
        inputs: &[],
        cache_key: &key,
    };
    assert!(executor
        .execute(request, &CancellationToken::new())
        .unwrap_err()
        .message()
        .contains("stage"));
}
