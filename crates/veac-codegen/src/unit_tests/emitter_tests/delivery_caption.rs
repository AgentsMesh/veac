use std::path::PathBuf;

use veac_artifact::ExecutionBindings;
use veac_codegen::emitter::{emit_all, BackendAction, BackendOutput, BackendPhase, BackendProduct};
use veac_plan::canonical::*;
use veac_plan::{ResolvedClipSource, ResolvedRenderPlan};

use super::support::{bindings, input_bindings, rekey_visual, resolved, text_fixture, time};

mod ass;
mod plain;

#[test]
fn caption_sidecars_render_sorted_exact_srt_and_webvtt_files() {
    for (format, expected) in [
        (
            CaptionSidecarFormat::Srt,
            "1\n00:00:00,500 --> 00:00:01,500\nFirst\n\n\
             2\n00:00:01,000 --> 00:00:02,000\nSecond\n\n",
        ),
        (
            CaptionSidecarFormat::WebVtt,
            "WEBVTT\n\n00:00:00.500 --> 00:00:01.500\nFirst\n\n\
             00:00:01.000 --> 00:00:02.000\nSecond\n\n",
        ),
    ] {
        let (plan, bindings, output) = caption_plan(format);
        let bundle = emit_all(&plan, &bindings).unwrap();
        assert_eq!(bundle.tasks().len(), 1);
        let task = &bundle.tasks()[0];
        assert_eq!(task.deliverable_id.as_str(), "dlv_caption");
        assert_eq!(task.phase, BackendPhase::Single);
        assert_eq!(task.product, BackendProduct::CaptionSidecar);
        assert_eq!(task.output, BackendOutput::File(output.clone()));
        let BackendAction::WriteFile { path, content } = &task.action else {
            panic!("caption sidecar must be a file write")
        };
        assert_eq!(path, &output);
        assert_eq!(content, expected.as_bytes());
    }
}

#[test]
fn caption_sidecars_fail_closed_for_missing_tracks_and_bindings() {
    let (mut plan, bindings, _) = caption_plan(CaptionSidecarFormat::Srt);
    let DeliverableKind::CaptionSidecar(settings) = &mut plan.output.deliverables[0].kind else {
        unreachable!()
    };
    settings.track_ids = vec![TrackId::new("trk_absent").unwrap()];
    let error = emit_all(&plan, &bindings).unwrap_err();
    assert!(error.diagnostics().iter().any(|value| {
        value.code == "PLAN_CAPTION_SIDECAR_INVALID"
            && value.object_id.as_deref() == Some("dlv_caption")
    }));

    let (plan, _, _) = caption_plan(CaptionSidecarFormat::WebVtt);
    let bindings = input_bindings(&plan);
    let error = emit_all(&plan, &bindings).unwrap_err();
    assert!(error.diagnostics().iter().any(|value| {
        value.code == "OUTPUT_BINDING_MISSING" && value.object_id.as_deref() == Some("dlv_caption")
    }));
}

#[test]
fn equal_time_caption_cues_use_stable_track_and_item_order() {
    let (mut plan, bindings, _) = caption_plan(CaptionSidecarFormat::Srt);
    let track = plan.sequences[0]
        .tracks
        .iter_mut()
        .find(|track| track.id.as_str() == "trk_caption")
        .unwrap();
    for clip in &mut track.clips {
        if matches!(clip.source, ResolvedClipSource::Caption { .. }) {
            clip.record_range.start = time(300);
        }
    }
    track.clips.sort_by(|left, right| {
        left.source_order
            .cmp(&right.source_order)
            .then_with(|| left.id.cmp(&right.id))
    });
    let bundle = emit_all(&plan, &bindings).unwrap();
    let BackendAction::WriteFile { content, .. } = &bundle.tasks()[0].action else {
        panic!("caption sidecar must be a file write")
    };
    let text = String::from_utf8(content.clone()).unwrap();
    assert!(text.find("Second").unwrap() < text.find("First").unwrap());
}

fn caption_plan(format: CaptionSidecarFormat) -> (ResolvedRenderPlan, ExecutionBindings, PathBuf) {
    let mut plan = resolved(&text_fixture(true));
    let track = plan.sequences[0]
        .tracks
        .iter_mut()
        .find(|track| track.id.as_str() == "trk_caption")
        .unwrap();
    set_text(&mut track.clips[0], "Second");
    let mut first = track.clips[0].clone();
    first.id = ItemId::new("itm_caption_first").unwrap();
    first.source_order = track.clips[0].source_order + 1;
    rekey_visual(first.visual.as_mut().unwrap(), "caption_first");
    first.record_range.start = time(300);
    set_text(&mut first, "First");
    track.clips.push(first);
    track.clips.sort_by(|left, right| {
        left.record_range
            .start
            .partial_cmp(&right.record_range.start)
            .unwrap()
            .then_with(|| left.source_order.cmp(&right.source_order))
            .then_with(|| left.id.cmp(&right.id))
    });

    let deliverable = Deliverable {
        id: DeliverableId::new("dlv_caption").unwrap(),
        file_name: match format {
            CaptionSidecarFormat::Srt => "captions.srt",
            CaptionSidecarFormat::WebVtt => "captions.vtt",
            CaptionSidecarFormat::Ass => "captions.ass",
        }
        .to_owned(),
        kind: DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
            format,
            track_ids: vec![track.id.clone()],
        }),
    };
    let output = PathBuf::from("/tmp").join(&deliverable.file_name);
    plan.output.deliverables = vec![deliverable];
    let bindings = bindings(&plan);
    (plan, bindings, output)
}

fn set_text(clip: &mut veac_plan::ResolvedClip, text: &str) {
    let ResolvedClipSource::Caption { content, .. } = &mut clip.source else {
        unreachable!()
    };
    content.text = text.to_owned();
}

fn content(bundle: &veac_codegen::emitter::BackendBundle) -> String {
    let BackendAction::WriteFile { content, .. } = &bundle.tasks()[0].action else {
        panic!()
    };
    String::from_utf8(content.clone()).unwrap()
}
