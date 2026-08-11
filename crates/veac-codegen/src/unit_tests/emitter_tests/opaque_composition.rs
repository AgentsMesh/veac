use veac_plan::canonical::*;
use veac_plan::ResolvedClipSource;

use super::support::{bindings, emit_video_command, fixture, resolved};

#[test]
fn straight_alpha_delivery_keeps_alpha_aware_source_over() {
    let mut plan = resolved(&fixture());
    make_straight_alpha(&mut plan);

    let graph = graph(&plan);
    assert!(graph.contains("color=c=black@0"), "{graph}");
    assert!(graph.contains("unpremultiply=planes=7"), "{graph}");
    assert!(
        graph.contains("blend=all_expr='B+A*(65535-B)/65535'"),
        "{graph}"
    );
    assert!(!graph.contains("maskedmerge=planes=7"), "{graph}");
}

#[test]
fn opaque_parent_does_not_make_a_nested_sequence_opaque() {
    let mut plan = resolved(&fixture());
    let mut child = plan.sequences[0].clone();
    child.id = SequenceId::new("seq_opaque_child").unwrap();
    child.tracks[0].id = TrackId::new("trk_opaque_child").unwrap();
    child.tracks[0].clips[0].id = ItemId::new("itm_opaque_child").unwrap();
    plan.sequences[0].tracks[0].clips[0].source = ResolvedClipSource::Sequence {
        sequence_id: child.id.clone(),
    };
    plan.sequences.insert(0, child);

    let graph = graph(&plan);
    assert_eq!(
        graph.matches("maskedmerge=planes=7").count(),
        1,
        "only the opaque parent may use the fast path: {graph}"
    );
    assert_eq!(
        graph
            .matches("blend=all_expr='B+A*(65535-B)/65535'")
            .count(),
        1,
        "the transparent child must preserve alpha: {graph}"
    );
}

fn make_straight_alpha(plan: &mut veac_plan::ResolvedRenderPlan) {
    plan.output.deliverables[0].target = DeliverableTarget::File {
        name: "output.mov".to_owned(),
    };
    let video = plan
        .output
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap();
    video.container = OutputFormat::Mov;
    video.audio = None;
    video.video = VideoOutput {
        codec: VideoCodec::ProRes,
        pixel_format: PixelFormat::Yuva444p10le,
        alpha: AlphaMode::Straight,
        color_space: None,
        rate_control: VideoRateControl::Lossless,
        gop_size: None,
        b_frames: None,
        profile: Some(VideoProfile::ProRes4444),
        level: None,
    };
}

fn graph(plan: &veac_plan::ResolvedRenderPlan) -> String {
    emit_video_command(plan, &bindings(plan))
        .unwrap()
        .filter_graph
        .unwrap()
}
