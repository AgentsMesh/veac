use veac_plan::canonical::*;

use super::support::{bindings, emit_video_command, fixture, resolved, time};

#[test]
fn layers_are_sorted_before_opaque_overlay_composition() {
    let mut plan = resolved(&fixture());
    let track = &mut plan.sequences[0].tracks[0];
    let mut copies = Vec::new();
    for (index, (id, start, z, opacity)) in [
        ("itm_layer_c", 300, 3, 0.3),
        ("itm_layer_a", 100, 1, 0.1),
        ("itm_layer_b", 100, 2, 0.2),
    ]
    .into_iter()
    .enumerate()
    {
        let mut clip = track.clips[0].clone();
        clip.id = ItemId::new(id).unwrap();
        clip.source_order = index as u32;
        clip.record_range.start = time(start);
        let visual = clip.visual.as_mut().unwrap();
        visual.compositing.z_index = z;
        visual.opacity = Animatable::constant(opacity);
        copies.push(clip);
    }
    track.clips = copies;
    track.clips.sort_by(|left, right| {
        left.record_range
            .start
            .partial_cmp(&right.record_range.start)
            .unwrap()
            .then(left.source_order.cmp(&right.source_order))
            .then(left.id.cmp(&right.id))
    });
    track.source_order = 2;
    let mut second_track = track.clone();
    second_track.id = TrackId::new("trk_second").unwrap();
    second_track.order = 1;
    second_track.source_order = 1;
    second_track.clips.truncate(1);
    second_track.clips[0].id = ItemId::new("itm_second").unwrap();
    second_track.clips[0].visual.as_mut().unwrap().opacity = Animatable::constant(0.4);
    plan.sequences[0].tracks.push(second_track);
    plan.sequences[0].duration = time(900);

    let graph = graph(&plan);
    let positions = ["*(0.1)'", "*(0.2)'", "*(0.3)'", "*(0.4)'"].map(|marker| {
        graph
            .find(marker)
            .unwrap_or_else(|| panic!("missing {marker}: {graph}"))
    });
    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "wrong layer order {positions:?}: {graph}"
    );
    assert_eq!(
        graph.matches("maskedmerge=planes=7:enable='gte(t,").count(),
        4,
        "graph={graph}"
    );
    assert!(!graph.contains("unpremultiply=planes=7"), "graph={graph}");
    assert_eq!(
        graph.matches("stop_mode=add:stop=1:color=black@0").count(),
        4,
        "every finite source-over layer needs transparent timeline padding: {graph}"
    );
}

fn graph(plan: &veac_plan::ResolvedRenderPlan) -> String {
    emit_video_command(plan, &bindings(plan))
        .unwrap()
        .filter_graph
        .unwrap()
}
