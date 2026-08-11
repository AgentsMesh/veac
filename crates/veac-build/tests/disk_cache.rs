mod support;

use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use veac_artifact::{
    ArtifactDependency, ArtifactDependencyRole, ArtifactDescriptor, ArtifactParameters,
    ArtifactStore, ProducedArtifactParameters, ProducerFingerprint, RenderOutputParameters,
};
use veac_build::*;

#[derive(Clone)]
struct CasExecutor {
    store: ArtifactStore,
    calls: Arc<AtomicUsize>,
    delay_ms: u64,
    missing: bool,
}

impl NodeExecutor<support::Action> for CasExecutor {
    fn implementation_identity(&self, _action: &support::Action) -> BuildResult<ContentDigest> {
        Ok(ContentDigest::sha256(b"cas-executor-test-v1"))
    }

    fn execute(
        &self,
        request: ExecuteRequest<'_, support::Action>,
        _cancellation: &CancellationToken,
    ) -> Result<ArtifactOutputs, ExecutionError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        thread::sleep(Duration::from_millis(self.delay_ms));
        if self.missing {
            return ArtifactOutputs::one(&support::output(), ContentDigest::sha256(b"missing"))
                .map_err(|error| ExecutionError::failed(error.to_string()));
        }
        let dependencies = request
            .inputs
            .iter()
            .map(|input| {
                ArtifactDependency::new(ArtifactDependencyRole::Input, input.digest.clone())
            })
            .collect();
        let descriptor = ArtifactDescriptor::new(
            ProducerFingerprint {
                name: "disk-cache-test".to_owned(),
                version: "1".to_owned(),
                configuration: request.cache_key.digest().clone(),
            },
            dependencies,
            ArtifactParameters::VideoMaster(ProducedArtifactParameters::Render(
                RenderOutputParameters::new(0, format!("{}.bin", request.node_id)),
            )),
        );
        let payload = format!("payload:{}", request.node_id);
        let record = self
            .store
            .put(&descriptor, payload.as_bytes())
            .map_err(|error| ExecutionError::failed(error.to_string()))?;
        ArtifactOutputs::one(&support::output(), record.key)
            .map_err(|error| ExecutionError::failed(error.to_string()))
    }
}

#[test]
fn disk_cache_persists_verified_artifacts_across_instances() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("artifacts"));
    let calls = Arc::new(AtomicUsize::new(0));
    let graph = graph();
    let executor = executor(&store, &calls, 0, false);
    let first_cache = cache(&store, temp.path());
    assert_eq!(first_cache.artifact_store().root(), store.root());
    let first = support::scheduler(1)
        .run(&graph, &executor, &first_cache, CancellationToken::new())
        .unwrap();
    let second_cache = cache(&store, temp.path());
    let second = support::scheduler(1)
        .run(&graph, &executor, &second_cache, CancellationToken::new())
        .unwrap();

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(first.nodes[0].status, NodeStatus::Executed);
    assert_eq!(second.nodes[0].status, NodeStatus::CacheHit);
    let artifact = second.nodes[0]
        .outputs
        .as_ref()
        .unwrap()
        .get(support::output().name())
        .unwrap();
    assert!(store.open(artifact).unwrap().is_some());
}

#[cfg(unix)]
#[test]
fn computation_lease_root_rejects_symlink_authority() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("artifacts"));
    let actual = temp.path().join("actual-leases");
    let link = temp.path().join("linked-leases");
    std::fs::create_dir(&actual).unwrap();
    std::os::unix::fs::symlink(&actual, &link).unwrap();

    let error = DiskBuildCache::new(store, link).err().unwrap();
    assert!(error.message().contains("non-symlink directory"));
}

#[test]
fn computation_record_rejects_missing_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("artifacts"));
    let calls = Arc::new(AtomicUsize::new(0));
    let receipt = support::scheduler(1)
        .run(
            &graph(),
            &executor(&store, &calls, 0, true),
            &cache(&store, temp.path()),
            CancellationToken::new(),
        )
        .unwrap();

    assert_eq!(receipt.outcome, BuildOutcome::Failed);
    assert!(receipt.nodes[0]
        .message
        .as_deref()
        .unwrap()
        .contains("referenced artifact"));
}

fn graph() -> ValidatedGraph<support::Action> {
    let mut builder = GraphBuilder::new();
    builder.add_node(support::node("render")).unwrap();
    builder.validate().unwrap()
}

fn cache(store: &ArtifactStore, root: &std::path::Path) -> DiskBuildCache {
    DiskBuildCache::new(store.clone(), root.join("leases")).unwrap()
}

fn executor(
    store: &ArtifactStore,
    calls: &Arc<AtomicUsize>,
    delay_ms: u64,
    missing: bool,
) -> CasExecutor {
    CasExecutor {
        store: store.clone(),
        calls: calls.clone(),
        delay_ms,
        missing,
    }
}
