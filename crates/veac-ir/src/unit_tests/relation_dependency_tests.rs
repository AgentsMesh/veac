use crate::MatteDependencyGraph;

#[test]
fn empty_and_branched_graphs_report_exact_depths() {
    assert_eq!(MatteDependencyGraph::<u32>::new().analyze().max_depth, 0);

    let mut graph = MatteDependencyGraph::new();
    graph.add("consumer", "left");
    graph.add("consumer", "right");
    graph.add("left", "root");
    graph.add("right", "root");
    let analysis = graph.analyze();
    assert!(!analysis.cyclic);
    assert_eq!(analysis.max_depth, 2);
}

#[test]
fn duplicate_edges_do_not_change_depth_and_cycles_are_detected() {
    let mut graph = MatteDependencyGraph::new();
    graph.add("a", "b");
    graph.add("a", "b");
    assert_eq!(graph.analyze().max_depth, 1);

    graph.add("b", "a");
    assert!(graph.analyze().cyclic);

    let mut self_cycle = MatteDependencyGraph::new();
    self_cycle.add("self", "self");
    assert!(self_cycle.analyze().cyclic);
}

#[test]
fn matte_depth_is_measured_in_edges_at_the_64_boundary() {
    let mut allowed = MatteDependencyGraph::new();
    for index in 0..64 {
        allowed.add(index, index + 1);
    }
    assert_eq!(allowed.analyze().max_depth, 64);

    let mut rejected = MatteDependencyGraph::new();
    for index in 0..65 {
        rejected.add(index, index + 1);
    }
    assert_eq!(rejected.analyze().max_depth, 65);
}
