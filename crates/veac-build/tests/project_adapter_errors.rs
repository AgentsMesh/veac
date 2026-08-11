mod project_support;

use project_support::{graph, Fixture};
use veac_project::{OutputId, TargetInstanceId};

#[test]
fn adapter_rejects_each_missing_edge_authority() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter();

    let mut missing_dependency = graph();
    missing_dependency.edges[0].dependency = TargetInstanceId::from("missing");
    assert!(adapter
        .adapt(&missing_dependency)
        .err()
        .unwrap()
        .message()
        .contains("dependency"));

    let mut missing_consumer = graph();
    missing_consumer.edges[0].consumer = TargetInstanceId::from("missing");
    assert!(adapter
        .adapt(&missing_consumer)
        .err()
        .unwrap()
        .message()
        .contains("consumer"));

    let mut removed_producer = graph();
    removed_producer.edges[0].dependency = removed_producer.edges[0].consumer.clone();
    assert!(adapter
        .adapt(&removed_producer)
        .err()
        .unwrap()
        .message()
        .contains("producer"));
}

#[test]
fn adapter_supports_order_edges_and_rejects_unknown_delivery_outputs() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter();
    let mut ordered = graph();
    ordered.edges[0].binding = None;
    ordered.edges[0].output = None;
    assert_eq!(adapter.adapt(&ordered).unwrap().graph.len(), 3);

    let mut missing_output = graph();
    missing_output.instances[0].deliveries[0].output = OutputId::from("missing");
    assert!(adapter
        .adapt(&missing_output)
        .err()
        .unwrap()
        .message()
        .contains("delivery output"));

    let mut duplicate = graph();
    duplicate.instances[1].id = duplicate.instances[0].id.clone();
    assert!(adapter
        .adapt(&duplicate)
        .err()
        .unwrap()
        .message()
        .contains("collision"));
}
