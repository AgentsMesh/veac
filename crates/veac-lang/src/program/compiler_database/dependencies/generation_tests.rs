use super::*;

#[test]
fn repeated_observation_rotates_admission_but_preserves_pending_generation() {
    let mut graph = DependencyGraph::new(usize::MAX, usize::MAX);
    let revision = CompilerSourceRevision::new("main.veac", "module {}");
    graph.observe("main.veac", revision);
    let first_observation = graph.observation_token("main.veac").unwrap();
    graph.invalidate(vec!["main.veac".into()]);
    let first_invalidation = graph.invalidation_generation("main.veac").unwrap();

    graph.observe("main.veac", revision);
    let second_observation = graph.observation_token("main.veac").unwrap();
    let second_invalidation = graph.invalidation_generation("main.veac").unwrap();

    assert!(!first_observation.matches(&second_observation));
    assert!(first_invalidation.matches(&second_invalidation));
}
