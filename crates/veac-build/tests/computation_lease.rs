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
    ArtifactDescriptor, ArtifactParameters, ArtifactStore, ProducedArtifactParameters,
    ProducerFingerprint, RenderOutputParameters,
};
use veac_build::*;

#[derive(Clone)]
struct SlowCas {
    store: ArtifactStore,
    calls: Arc<AtomicUsize>,
}

impl NodeExecutor<support::Action> for SlowCas {
    fn implementation_identity(&self, _action: &support::Action) -> BuildResult<ContentDigest> {
        Ok(ContentDigest::sha256(b"slow-cas-test-v1"))
    }

    fn execute(
        &self,
        request: ExecuteRequest<'_, support::Action>,
        _cancellation: &CancellationToken,
    ) -> Result<ArtifactOutputs, ExecutionError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        thread::sleep(Duration::from_millis(80));
        let descriptor = ArtifactDescriptor::new(
            ProducerFingerprint {
                name: "lease-test".to_owned(),
                version: "1".to_owned(),
                configuration: request.cache_key.digest().clone(),
            },
            Vec::new(),
            ArtifactParameters::VideoMaster(ProducedArtifactParameters::Render(
                RenderOutputParameters::new(0, "result.bin"),
            )),
        );
        let record = self
            .store
            .put(&descriptor, b"result")
            .map_err(|error| ExecutionError::failed(error.to_string()))?;
        ArtifactOutputs::one(&support::output(), record.key)
            .map_err(|error| ExecutionError::failed(error.to_string()))
    }
}

#[test]
fn independent_cache_instances_share_one_computation_lease() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("artifacts"));
    let calls = Arc::new(AtomicUsize::new(0));
    let graph = Arc::new(graph());
    let executor = Arc::new(SlowCas {
        store: store.clone(),
        calls: calls.clone(),
    });
    let root = temp.path().join("leases");
    let workers = (0..2)
        .map(|_| {
            let graph = graph.clone();
            let executor = executor.clone();
            let cache = DiskBuildCache::new(store.clone(), &root).unwrap();
            thread::spawn(move || {
                support::scheduler(1)
                    .run(&graph, executor.as_ref(), &cache, CancellationToken::new())
                    .unwrap()
            })
        })
        .collect::<Vec<_>>();
    let receipts = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let mut statuses = receipts
        .iter()
        .map(|receipt| receipt.nodes[0].status)
        .collect::<Vec<_>>();
    statuses.sort_by_key(|status| match status {
        NodeStatus::Executed => 0,
        NodeStatus::CacheHit => 1,
        _ => 2,
    });
    assert_eq!(statuses, [NodeStatus::Executed, NodeStatus::CacheHit]);
}

fn graph() -> ValidatedGraph<support::Action> {
    let mut builder = GraphBuilder::new();
    builder.add_node(support::node("render")).unwrap();
    builder.validate().unwrap()
}
