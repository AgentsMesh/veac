use veac_codegen::emitter::{
    emit_all, BackendAction, MAX_FILTER_GRAPH_BYTES, MAX_INLINE_FILTER_GRAPH_BYTES,
};
use veac_plan::canonical::*;
use veac_plan::ResolvedClipSource;

use super::support::{bindings, rekey_visual, resolved, text_fixture};

#[test]
fn auxiliary_visual_deliverables_emit_large_bounded_script_eligible_graphs() {
    for (file, kind) in [
        (
            "frames-%d.png",
            DeliverableKind::ImageSequence(ImageSequenceOutput {
                format: ImageFormat::Png,
                start_number: 1,
            }),
        ),
        (
            "scope.png",
            DeliverableKind::Scope(ScopeOutput {
                scope: VideoScope::Waveform,
                at: RationalTime::zero(600).unwrap(),
                width: 320,
                height: 180,
                format: ImageFormat::Png,
            }),
        ),
    ] {
        let mut plan = oversized_visual_plan();
        let id = DeliverableId::new("dlv_aux_limit").unwrap();
        let target = if matches!(kind, DeliverableKind::ImageSequence(_)) {
            DeliverableTarget::ImageSequence {
                pattern: file.to_owned(),
            }
        } else {
            DeliverableTarget::File {
                name: file.to_owned(),
            }
        };
        plan.output.deliverables = vec![Deliverable {
            id: id.clone(),
            target,
            kind,
        }];
        let local = bindings(&plan);

        let bundle = emit_all(&plan, &local).unwrap();
        let command = bundle.tasks().iter().find_map(|task| match &task.action {
            BackendAction::Ffmpeg(command) => Some(command),
            BackendAction::WriteFile { .. } => None,
        });
        let bytes = command.unwrap().filter_graph.as_ref().unwrap().len();
        assert!(bytes > MAX_INLINE_FILTER_GRAPH_BYTES, "graph bytes={bytes}");
        assert!(bytes <= MAX_FILTER_GRAPH_BYTES, "graph bytes={bytes}");
    }
}

fn oversized_visual_plan() -> veac_plan::ResolvedRenderPlan {
    let mut plan = resolved(&text_fixture(false));
    let base = plan.sequences[0].tracks[1].clone();
    plan.sequences[0].tracks.truncate(1);
    for index in 0..3 {
        let mut track = base.clone();
        track.id = TrackId::new(format!("trk_script_graph_{index}")).unwrap();
        track.order = 10 + index;
        track.source_order = 1 + index as u32;
        track.clips[0].id = ItemId::new(format!("itm_script_graph_{index}")).unwrap();
        rekey_visual(
            track.clips[0].visual.as_mut().unwrap(),
            &format!("script_graph_{index}"),
        );
        let ResolvedClipSource::Text { content } = &mut track.clips[0].source else {
            panic!("text fixture")
        };
        content.text = "a".repeat(40_000);
        content.styled_mut().unwrap().background = None;
        plan.sequences[0].tracks.push(track);
    }
    plan
}
