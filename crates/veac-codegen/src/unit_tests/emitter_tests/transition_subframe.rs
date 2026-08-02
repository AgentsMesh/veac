use veac_plan::canonical::{Rational, TransitionAlignment, TransitionKind};

use super::support::time;
use super::transitions::{graph, transition_plan};

#[test]
fn outgoing_handle_without_a_frame_is_seeded_from_the_last_real_frame() {
    let mut plan = transition_plan(TransitionAlignment::Centered, TransitionKind::Dissolve);
    plan.sequences[0].settings.frame_rate = Rational::new(12, 1).unwrap();
    let transition = &mut plan.sequences[0].tracks[0].transitions[0];
    transition.record_window.start = time(588);
    transition.record_window.duration = time(24);
    transition.outgoing_handle.offset = time(588);
    transition.outgoing_handle.duration = time(12);
    transition.incoming_handle.duration = time(12);
    let graph = graph(&plan);

    assert!(graph.contains("trim=start=0.833333333333,reverse,trim=end_frame=1"));
    assert!(!graph.contains("trim=start=0.98:duration=0.02"));
    assert!(graph.contains("xfade=transition=fade:duration=0.083333333333"));
}

#[test]
fn transition_output_gets_monotonic_frame_pts_before_hold_and_offset() {
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
    assert_edge(&graph, held, "setpts=PTS+0.9/TB", offset);
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
