mod support;

use veac_build::*;

use support::{input, node, output, Action, FakeExecutor};

#[test]
fn cache_hits_skip_execution_and_preserve_deterministic_receipts() {
    let graph = pipeline(1, ResourceClaim::default());
    let cache = MemoryBuildCache::new();
    let first_executor = FakeExecutor::default();
    let first = support::scheduler(2)
        .run(&graph, &first_executor, &cache, CancellationToken::new())
        .unwrap();
    let second_executor = FakeExecutor::default();
    let second = support::scheduler(2)
        .run(&graph, &second_executor, &cache, CancellationToken::new())
        .unwrap();

    assert_eq!(first.outcome, BuildOutcome::Succeeded);
    assert_eq!(cache.len().unwrap(), 4);
    assert!(first
        .nodes
        .iter()
        .all(|receipt| receipt.status == NodeStatus::Executed));
    assert!(second
        .nodes
        .iter()
        .all(|receipt| receipt.status == NodeStatus::CacheHit));
    assert_eq!(second_executor.stats.max_active(), 0);
    for (left, right) in first.nodes.iter().zip(&second.nodes) {
        assert_eq!(left.node_id, right.node_id);
        assert_eq!(left.cache_key, right.cache_key);
        assert_eq!(left.outputs, right.outputs);
    }
}

#[test]
fn changing_one_action_invalidates_only_it_and_its_descendants() {
    let cache = MemoryBuildCache::new();
    let baseline = pipeline(1, ResourceClaim::default());
    support::scheduler(2)
        .run(
            &baseline,
            &FakeExecutor::default(),
            &cache,
            CancellationToken::new(),
        )
        .unwrap();
    let revised = pipeline(2, ResourceClaim::default());
    let executor = FakeExecutor::default();
    let receipt = support::scheduler(2)
        .run(&revised, &executor, &cache, CancellationToken::new())
        .unwrap();

    assert_ne!(baseline.digest(), revised.digest());
    assert_eq!(status(&receipt, "source"), NodeStatus::CacheHit);
    assert_eq!(status(&receipt, "independent"), NodeStatus::CacheHit);
    assert_eq!(status(&receipt, "middle"), NodeStatus::Executed);
    assert_eq!(status(&receipt, "final"), NodeStatus::Executed);
    assert_eq!(executor.stats.count("source"), 0);
    assert_eq!(executor.stats.count("independent"), 0);
    assert_eq!(executor.stats.count("middle"), 1);
    assert_eq!(executor.stats.count("final"), 1);
}

#[test]
fn scheduling_resources_change_graph_identity_but_reuse_artifacts() {
    let cache = MemoryBuildCache::new();
    let baseline = pipeline(1, ResourceClaim::default());
    support::scheduler(2)
        .run(
            &baseline,
            &FakeExecutor::default(),
            &cache,
            CancellationToken::new(),
        )
        .unwrap();
    let resourced = pipeline(1, ResourceClaim::new(1, 32, 0));
    let receipt = support::scheduler(2)
        .run(
            &resourced,
            &FakeExecutor::default(),
            &cache,
            CancellationToken::new(),
        )
        .unwrap();

    assert_ne!(baseline.digest(), resourced.digest());
    assert!(receipt
        .nodes
        .iter()
        .all(|node| node.status == NodeStatus::CacheHit));
}

#[derive(Clone, Copy)]
enum Fault {
    Lookup,
    Store,
    Corrupt,
}

struct FaultCache(Fault);

impl BuildCache for FaultCache {
    fn get(&self, _key: &NodeCacheKey) -> BuildResult<Option<ArtifactOutputs>> {
        match self.0 {
            Fault::Lookup => Err(BuildError::cache("lookup fault")),
            Fault::Store => Ok(None),
            Fault::Corrupt => Ok(Some(ArtifactOutputs::one(
                &OutputSlot::<support::Media>::new("wrong").unwrap(),
                ContentDigest::sha256(b"wrong"),
            )?)),
        }
    }

    fn put(&self, _key: &NodeCacheKey, _outputs: &ArtifactOutputs) -> BuildResult<()> {
        match self.0 {
            Fault::Store => Err(BuildError::cache("store fault")),
            _ => Ok(()),
        }
    }
}

#[test]
fn cache_lookup_store_and_corruption_fail_the_node() {
    for (fault, fragment) in [
        (Fault::Lookup, "lookup failed"),
        (Fault::Store, "publication failed"),
        (Fault::Corrupt, "do not match"),
    ] {
        let graph = one_node();
        let receipt = support::scheduler(1)
            .run(
                &graph,
                &FakeExecutor::default(),
                &FaultCache(fault),
                CancellationToken::new(),
            )
            .unwrap();
        assert_eq!(receipt.outcome, BuildOutcome::Failed);
        assert_eq!(receipt.nodes[0].status, NodeStatus::Failed);
        assert!(receipt.nodes[0]
            .message
            .as_deref()
            .unwrap()
            .contains(fragment));
    }
}

fn pipeline(middle_revision: u32, source_resources: ResourceClaim) -> ValidatedGraph<Action> {
    let source = node("source").resources(source_resources);
    let middle_ref = source.output_ref(&output());
    let mut middle_action = Action::new("middle");
    middle_action.revision = middle_revision;
    let middle = NodeSpec::new(support::id("middle"), middle_action)
        .output(&output())
        .bind(&input(), &middle_ref);
    let final_node = node("final").bind(&input(), &middle.output_ref(&output()));
    let independent = node("independent");
    let mut builder = GraphBuilder::new();
    for node in [final_node, independent, middle, source] {
        builder.add_node(node).unwrap();
    }
    builder.validate().unwrap()
}

fn one_node() -> ValidatedGraph<Action> {
    let mut builder = GraphBuilder::new();
    builder.add_node(node("one")).unwrap();
    builder.validate().unwrap()
}

fn status(receipt: &BuildReceipt, id: &str) -> NodeStatus {
    receipt.node(&support::id(id)).unwrap().status
}
