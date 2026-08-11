mod support;

use veac_artifact::{ContentDigest, DigestAlgorithm};
use veac_build::*;

use support::Media;

#[test]
fn identifiers_and_typed_ports_enforce_closed_names() {
    let id = NodeId::new("plate-01.main").unwrap();
    let input = InputSlot::<Media>::new("video_in").unwrap();
    let output = OutputSlot::<Media>::new("video-out").unwrap();
    let spec = NodeSpec::new(id.clone(), support::Action::new("plate")).output(&output);
    let reference = spec.output_ref(&output);

    assert_eq!(id.as_str(), "plate-01.main");
    assert_eq!(id.to_string(), "plate-01.main");
    assert_eq!(input.name().as_str(), "video_in");
    assert_eq!(output.name().to_string(), "video-out");
    assert_eq!(reference.node(), &id);
    assert_eq!(reference.output(), output.name());
    assert_eq!(reference.clone(), reference);
    assert!(format!("{reference:?}").contains("plate-01.main"));
    assert!(NodeId::new("").is_err());
    assert!(NodeId::new("contains space").is_err());
    assert!(NodeId::new("x".repeat(129)).is_err());
    assert!(PortName::new("/").is_err());
    assert!(PortName::new("x".repeat(65)).is_err());
}

#[test]
fn output_sets_validate_identity_and_uniqueness() {
    let slot = OutputSlot::<Media>::new("main").unwrap();
    let digest = ContentDigest::sha256(b"payload");
    let outputs = ArtifactOutputs::one(&slot, digest.clone()).unwrap();

    assert_eq!(outputs.len(), 1);
    assert!(!outputs.is_empty());
    assert_eq!(outputs.get(slot.name()), Some(&digest));
    assert_eq!(outputs.iter().next().unwrap().0, slot.name());
    assert!(ArtifactOutputs::try_from_iter(Vec::new())
        .unwrap()
        .is_empty());
    assert!(ArtifactOutputs::try_from_iter([
        (slot.name().clone(), digest.clone()),
        (slot.name().clone(), digest),
    ])
    .is_err());

    let invalid = ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: "not-a-digest".to_owned(),
    };
    assert!(ArtifactOutputs::one(&slot, invalid).is_err());
}

#[test]
fn public_errors_cancellation_and_limits_are_inspectable() {
    let error = BuildError::cache("broken cache");
    assert_eq!(error.kind(), BuildErrorKind::Cache);
    assert_eq!(error.message(), "broken cache");
    assert_eq!(error.to_string(), "broken cache");
    assert_eq!(
        BuildError::invalid("bad").kind(),
        BuildErrorKind::InvalidContract
    );

    let token = CancellationToken::new();
    assert!(!token.is_cancelled());
    token.clone().cancel();
    assert!(token.is_cancelled());

    assert!(BuildLimits::new(0, ResourceClaim::new(1, 0, 0)).is_err());
    assert!(BuildLimits::new(1, ResourceClaim::new(0, 0, 0)).is_err());
    let limits = BuildLimits::new(2, ResourceClaim::new(3, 512, 1)).unwrap();
    assert_eq!(limits.jobs, 2);
    assert_eq!(ResourceClaim::default(), ResourceClaim::new(1, 0, 0));
}

#[test]
fn execution_errors_and_node_statuses_preserve_meaning() {
    let failed = ExecutionError::failed("failed");
    let cancelled = ExecutionError::cancelled("cancelled");
    assert_eq!(failed.kind(), ExecutionErrorKind::Failed);
    assert_eq!(failed.message(), "failed");
    assert_eq!(failed.to_string(), "failed");
    assert_eq!(cancelled.kind(), ExecutionErrorKind::Cancelled);
    assert!(NodeStatus::Executed.is_success());
    assert!(NodeStatus::CacheHit.is_success());
    assert!(!NodeStatus::Failed.is_success());
}

#[test]
fn memory_and_null_caches_follow_the_same_contract() {
    let memory = MemoryBuildCache::new();
    let null = NullBuildCache;
    let graph = {
        let mut builder = GraphBuilder::new();
        builder.add_node(support::node("one")).unwrap();
        builder.validate().unwrap()
    };
    let executor = support::FakeExecutor::default();
    let first = support::scheduler(1)
        .run(&graph, &executor, &memory, CancellationToken::new())
        .unwrap();
    let key = first.nodes[0].cache_key.as_ref().unwrap();

    assert_eq!(key.digest().value.len(), 64);
    assert_eq!(memory.len().unwrap(), 1);
    assert!(!memory.is_empty().unwrap());
    assert!(memory.get(key).unwrap().is_some());
    assert!(null.get(key).unwrap().is_none());
    null.put(key, first.nodes[0].outputs.as_ref().unwrap())
        .unwrap();
}
