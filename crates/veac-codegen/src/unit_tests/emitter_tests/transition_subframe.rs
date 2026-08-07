use veac_plan::canonical::{Rational, TransitionAlignment, TransitionKind};

use super::transitions::{graph, transition_plan};

#[test]
fn transition_output_gets_monotonic_frame_pts_before_bounded_composite() {
    let mut plan = transition_plan(
        TransitionAlignment::Centered,
        TransitionKind::Wipe {
            direction: veac_plan::canonical::CardinalDirection::Left,
            angle_degrees: 0.0,
            softness: 0.1,
        },
    );
    plan.sequences[0].settings.frame_rate = Rational::new(30_000, 1_001).unwrap();
    let graph = graph(&plan);
    let mixed = first_label(&graph, "transitionv");
    let rebased = first_label(&graph, "transitionrebasev");
    let held = first_label(&graph, "transitionholdv");
    let offset = first_label(&graph, "transitionoffsetv");

    assert!(
        graph.contains(&format!("format=rgba[{mixed}]")),
        "graph={graph}"
    );
    assert_edge(&graph, mixed, "setpts=N*1001/(30000*TB)", rebased);
    assert_edge(&graph, rebased, "tpad=stop_mode=clone:stop=-1", held);
    assert_edge(&graph, held, "setpts=PTS+0.8/TB", offset);
}

fn first_label<'a>(graph: &'a str, prefix: &str) -> &'a str {
    let start = graph.find(&format!("[{prefix}")).unwrap() + 1;
    let end = start + graph[start..].find(']').unwrap();
    &graph[start..end]
}

fn assert_edge(graph: &str, input: &str, filter: &str, output: &str) {
    let edge = format!("[{input}]{filter}[{output}]");
    assert!(graph.contains(&edge), "missing {edge}: {graph}");
}
