mod support;

use veac_build::*;

#[derive(Debug, Clone)]
enum BadAction {
    Kind,
    Version,
    Huge,
    Error,
}

impl BuildAction for BadAction {
    fn kind(&self) -> &str {
        match self {
            Self::Kind => "invalid/kind",
            _ => "bad-action",
        }
    }

    fn version(&self) -> u32 {
        if matches!(self, Self::Version) {
            0
        } else {
            1
        }
    }

    fn canonical_bytes(&self) -> BuildResult<Vec<u8>> {
        match self {
            Self::Huge => Ok(vec![0; MAX_ACTION_BYTES + 1]),
            Self::Error => Err(BuildError::invalid("cannot canonicalize")),
            _ => Ok(Vec::new()),
        }
    }
}

#[test]
fn empty_graph_missing_outputs_and_zero_cpu_are_rejected() {
    let empty = GraphBuilder::<support::Action>::new()
        .validate()
        .unwrap_err();
    assert_eq!(empty.kind(), BuildErrorKind::ResourceLimit);

    let no_output = NodeSpec::new(support::id("empty"), support::Action::new("empty"));
    assert_error([no_output], "must declare between");

    let zero_cpu = support::node("zero").resources(ResourceClaim::new(0, 0, 0));
    assert_error([zero_cpu], "at least one CPU");
}

#[test]
fn node_output_and_graph_node_budgets_are_enforced() {
    let mut crowded = NodeSpec::new(support::id("crowded"), support::Action::new("crowded"));
    for index in 0..=MAX_NODE_OUTPUTS {
        crowded = crowded.output(&OutputSlot::<support::Media>::new(format!("o{index}")).unwrap());
    }
    assert_error([crowded], "must declare between");

    let mut builder = GraphBuilder::new();
    for index in 0..=MAX_GRAPH_NODES {
        builder
            .add_node(NodeSpec::new(
                NodeId::new(format!("n{index}")).unwrap(),
                support::Action::new("node"),
            ))
            .unwrap();
    }
    assert_eq!(
        builder.validate().unwrap_err().kind(),
        BuildErrorKind::ResourceLimit
    );
}

#[test]
fn graph_edge_budget_is_enforced_before_execution() {
    let source = support::node("source");
    let reference = source.output_ref(&support::output());
    let mut consumer = support::node("consumer");
    for index in 0..=MAX_GRAPH_EDGES {
        consumer = consumer.bind(
            &InputSlot::<support::Media>::new(format!("i{index}")).unwrap(),
            &reference,
        );
    }
    assert_error([source, consumer], "edge budget");
}

#[test]
fn action_contract_rejects_invalid_kind_version_size_and_errors() {
    for (action, fragment, kind) in [
        (
            BadAction::Kind,
            "action kind",
            BuildErrorKind::InvalidContract,
        ),
        (
            BadAction::Version,
            "version must be positive",
            BuildErrorKind::InvalidContract,
        ),
        (
            BadAction::Huge,
            "byte budget",
            BuildErrorKind::ResourceLimit,
        ),
        (
            BadAction::Error,
            "cannot canonicalize",
            BuildErrorKind::InvalidContract,
        ),
    ] {
        let mut builder = GraphBuilder::new();
        builder
            .add_node(
                NodeSpec::new(NodeId::new("bad").unwrap(), action)
                    .output(&OutputSlot::<support::Media>::new("out").unwrap()),
            )
            .unwrap();
        let error = builder.validate().unwrap_err();
        assert_eq!(error.kind(), kind);
        assert!(error.to_string().contains(fragment), "{error}");
    }
}

fn assert_error<const N: usize>(nodes: [NodeSpec<support::Action>; N], fragment: &str) {
    let mut builder = GraphBuilder::new();
    for node in nodes {
        builder.add_node(node).unwrap();
    }
    let error = builder.validate().unwrap_err();
    assert!(error.to_string().contains(fragment), "{error}");
}
