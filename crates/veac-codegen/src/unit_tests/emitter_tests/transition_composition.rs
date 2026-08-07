use veac_plan::canonical::*;
use veac_plan::{ResolvedApply, ResolvedApplyTarget, ResolvedClipSource, ResolvedRenderPlan};

use super::composition_advanced::{blur_stage, default_mix};
use super::support::time;
use super::transitions::{graph, transition_plan};

#[test]
fn translucent_endpoints_are_not_composited_below_the_transition() {
    let mut plan = transition_plan(TransitionAlignment::Centered, TransitionKind::Dissolve);
    for clip in &mut plan.sequences[0].tracks[0].clips {
        let visual = clip.visual.as_mut().unwrap();
        visual.opacity = Animatable::constant(0.5);
        visual.masks.push(circle_mask());
    }

    let graph = graph(&plan);
    let normal = nodes(&graph, "normalclipv");
    assert_eq!(normal.len(), 2, "graph={graph}");
    assert!(normal
        .iter()
        .any(|node| node.contains("start=0:duration=0.8")));
    assert!(normal
        .iter()
        .any(|node| node.contains("start=0.2:duration=0.8")));
    assert_eq!(window_uses(&graph, 0.0, 0.8), 1, "graph={graph}");
    assert_eq!(window_uses(&graph, 0.8, 1.0), 1, "graph={graph}");
    assert_eq!(window_uses(&graph, 1.0, 1.8), 1, "graph={graph}");
    assert!(graph.matches("[opacityv").count() >= 4, "graph={graph}");
    assert!(graph.matches("[maskv").count() >= 4, "graph={graph}");
}

#[test]
fn middle_clip_keeps_only_the_gap_between_two_transition_windows() {
    let plan = chained_plan(960);
    assert!(dynamic_media_endpoints(&plan));
    let graph = graph(&plan);
    let normal = nodes(&graph, "normalclipv");
    assert_eq!(normal.len(), 3, "graph={graph}");
    assert!(normal
        .iter()
        .any(|node| node.contains("start=0.2:duration=0.6")));
    assert_eq!(window_uses(&graph, 1.0, 1.6), 1, "graph={graph}");
    assert_eq!(
        graph.matches("xfade=transition=fade:duration=0.2").count(),
        2
    );
    assert_real_endpoint_trims(&graph, 4);
}

#[test]
fn touching_transition_windows_skip_an_empty_middle_normal_window() {
    let plan = chained_plan(600);
    let graph = graph(&plan);
    assert_eq!(nodes(&graph, "normalclipv").len(), 2, "graph={graph}");
    assert_eq!(graph.matches("xfade=transition=fade").count(), 2);
}

#[test]
fn layer_apply_processes_the_same_partitioned_transition_composition() {
    let mut plan = transition_plan(TransitionAlignment::Centered, TransitionKind::Dissolve);
    let sequence = &mut plan.sequences[0];
    let range = TimeRange::new(time(0), sequence.duration).unwrap();
    let track_id = sequence.tracks[0].id.clone();
    let item_ids = sequence.tracks[0]
        .clips
        .iter()
        .map(|clip| clip.id.clone())
        .collect();
    sequence.applies.push(ResolvedApply {
        id: ApplyId::new("apl_transition_layer").unwrap(),
        source_order: 0,
        record_range: range,
        target: ResolvedApplyTarget::Layer {
            track_id,
            item_ids,
            active_ranges: vec![range],
        },
        stages: vec![blur_stage("aps_transition_layer", range)],
        mix: default_mix(),
        matte: None,
    });

    let graph = graph(&plan);
    let normal = nodes(&graph, "normalclipv");
    assert_eq!(normal.len(), 4, "main and apply-band partitions: {graph}");
    assert_eq!(
        normal
            .iter()
            .filter(|node| node.contains("start=0:duration=0.8"))
            .count(),
        2
    );
    assert_eq!(
        normal
            .iter()
            .filter(|node| node.contains("start=0.2:duration=0.8"))
            .count(),
        2
    );
    assert_eq!(graph.matches("xfade=transition=fade").count(), 2);
    assert_eq!(window_uses(&graph, 0.8, 1.0), 2, "graph={graph}");
    assert!(graph.contains("gblur@"), "graph={graph}");
    assert!(graph.contains("applyreplacev"), "graph={graph}");
}

fn chained_plan(second_start: i64) -> ResolvedRenderPlan {
    let mut plan = transition_plan(TransitionAlignment::Centered, TransitionKind::Dissolve);
    let track = &mut plan.sequences[0].tracks[0];
    let mut third = track.clips[1].clone();
    third.id = ItemId::new("itm_transition_third").unwrap();
    third.source_order = 2;
    third.record_range.start = time(second_start);
    let duration = 1_080 - second_start;
    let mut second = track.transitions[0].clone();
    second.relation_id = RelationId::new("rel_transition_second").unwrap();
    second.outgoing_clip_id = track.clips[1].id.clone();
    second.incoming_clip_id = third.id.clone();
    second.cut_time = time(second_start + duration / 2);
    second.record_window = TimeRange::new(time(second_start), time(duration)).unwrap();
    second.outgoing_range = TimeRange::new(time(second_start - 480), time(duration)).unwrap();
    second.incoming_range = TimeRange::new(time(0), time(duration)).unwrap();
    track.clips.push(third);
    track.transitions.push(second);
    plan.sequences[0].duration = time(second_start + 600);
    plan
}

fn dynamic_media_endpoints(plan: &ResolvedRenderPlan) -> bool {
    plan.sequences[0].tracks[0].clips.iter().all(|clip| {
        matches!(
            &clip.source,
            ResolvedClipSource::Media {
                video_stream: Some(_),
                ..
            }
        )
    })
}

fn assert_real_endpoint_trims(graph: &str, expected: usize) {
    let mut endpoints = nodes(graph, "transitionoutv");
    endpoints.extend(nodes(graph, "transitioninv"));
    assert_eq!(endpoints.len(), expected, "graph={graph}");
    assert!(endpoints.iter().all(|node| {
        node.contains("trim=start=")
            && node.contains("duration=0.2,setpts=PTS-STARTPTS")
            && !node.contains("tpad")
    }));
}

fn nodes<'a>(graph: &'a str, output: &str) -> Vec<&'a str> {
    graph
        .split(';')
        .filter(|node| {
            node.rsplit_once('[')
                .is_some_and(|(_, label)| label.starts_with(output))
        })
        .collect()
}

fn window_uses(graph: &str, start: f64, end: f64) -> usize {
    graph
        .matches(&format!("gte(T\\,{start})*lt(T\\,{end})"))
        .count()
}

fn circle_mask() -> Mask {
    Mask {
        shape: MaskShape::Circle,
        position: Animatable::constant(Vec2 { x: 0.5, y: 0.5 }),
        scale: Animatable::constant(Vec2 { x: 0.8, y: 0.8 }),
        rotation_degrees: Animatable::constant(0.0),
        invert: false,
        feather_pixels: Animatable::constant(0.0),
        expansion_pixels: Animatable::constant(0.0),
    }
}
