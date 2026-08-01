use std::collections::BTreeMap;
use veac_artifact::ExecutionBindings;
use veac_codegen::emitter::{emit_all, BackendAction, BackendProduct};
use veac_plan::canonical::*;

use super::support::{bindings, resolved, text_fixture};

mod contracts;

#[test]
fn emits_every_deliverable_as_an_independent_typed_task() {
    let mut plan = resolved(&text_fixture(true));
    let caption_track = plan.sequences[0]
        .tracks
        .iter_mut()
        .find(|track| track.id.as_str() == "trk_caption")
        .unwrap();
    caption_track.state.visual_enabled = false;
    caption_track.state.include_in_render = true;
    caption_track.routing.visual = None;
    let video = plan.output.deliverables.remove(0);
    plan.output.deliverables = vec![
        deliverable(
            "dlv_caption",
            "captions.vtt",
            DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
                format: CaptionSidecarFormat::WebVtt,
                track_ids: vec![TrackId::new("trk_caption").unwrap()],
            }),
        ),
        deliverable(
            "dlv_frames",
            "frame-%d.png",
            DeliverableKind::ImageSequence(ImageSequenceOutput {
                format: ImageFormat::Png,
                start_number: 1001,
            }),
        ),
        video,
        deliverable(
            "dlv_stem",
            "master.wav",
            DeliverableKind::AudioStem(AudioStemOutput {
                format: AudioStemFormat::Wav,
                audio: pcm(),
                source: AudioMixSource::Master,
            }),
        ),
        deliverable(
            "dlv_waveform",
            "waveform.png",
            DeliverableKind::Scope(scope(VideoScope::Waveform)),
        ),
    ];
    plan.output
        .deliverables
        .sort_by(|left, right| left.id.cmp(&right.id));
    let bindings = all_bindings(&plan);
    let bundle = emit_all(&plan, &bindings).unwrap();
    contracts::assert_five_artifact_contract(&plan, &bindings, &bundle);
    assert_eq!(
        bundle.plan_identity().value,
        veac_plan::plan_hash(&plan).unwrap()
    );
    assert_eq!(bundle.tasks().len(), 5);
    let products: BTreeMap<_, _> = bundle
        .tasks()
        .iter()
        .map(|task| (task.deliverable_id.as_str(), task.product))
        .collect();
    assert_eq!(products["dlv_caption"], BackendProduct::CaptionSidecar);
    assert_eq!(products["dlv_frames"], BackendProduct::ImageSequence);
    assert_eq!(products["dlv_main"], BackendProduct::VideoMaster);
    assert_eq!(products["dlv_stem"], BackendProduct::AudioStem);
    assert_eq!(products["dlv_waveform"], BackendProduct::VideoWaveform);

    let caption = bundle
        .tasks()
        .iter()
        .find(|task| task.deliverable_id.as_str() == "dlv_caption")
        .unwrap();
    let BackendAction::WriteFile { content, .. } = &caption.action else {
        panic!("caption must be a deterministic write task")
    };
    let content = String::from_utf8(content.clone()).unwrap();
    assert!(content.starts_with("WEBVTT\n\n"));
    assert!(content.contains("00:00:01.000 --> 00:00:02.000"));
    assert!(content.contains("a'b:c%d\\e\nf"));

    let frames = ffmpeg(&bundle, "dlv_frames");
    assert!(pair(&frames.output_args, "-start_number", "1001"));
    assert!(pair(&frames.output_args, "-pix_fmt", "rgba"));
    let stem = ffmpeg(&bundle, "dlv_stem");
    assert!(pair(&stem.output_args, "-c:a", "pcm_s16le"));
    let scope = ffmpeg(&bundle, "dlv_waveform");
    assert!(scope
        .filter_graph
        .as_deref()
        .unwrap()
        .contains("waveform=mode=column"));
}

#[test]
fn missing_binding_fails_closed_for_the_specific_deliverable() {
    let mut plan = resolved(&text_fixture(true));
    let bindings = bindings(&plan);
    plan.output.deliverables.push(deliverable(
        "dlv_frames",
        "frame-%d.png",
        DeliverableKind::ImageSequence(ImageSequenceOutput {
            format: ImageFormat::Png,
            start_number: 1,
        }),
    ));
    plan.output
        .deliverables
        .sort_by(|left, right| left.id.cmp(&right.id));
    let error = emit_all(&plan, &bindings).unwrap_err();
    assert!(error
        .diagnostics()
        .iter()
        .any(|value| value.code == "OUTPUT_BINDING_MISSING"
            && value.object_id.as_deref() == Some("dlv_frames")));
}

fn all_bindings(plan: &veac_plan::ResolvedRenderPlan) -> ExecutionBindings {
    bindings(plan)
}

fn deliverable(id: &str, file_name: &str, kind: DeliverableKind) -> Deliverable {
    let target = match &kind {
        DeliverableKind::ImageSequence(_) => DeliverableTarget::ImageSequence {
            pattern: file_name.to_owned(),
        },
        _ => DeliverableTarget::File {
            name: file_name.to_owned(),
        },
    };
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        target,
        kind,
    }
}

fn pcm() -> AudioOutput {
    AudioOutput {
        codec: AudioCodec::PcmS16Le,
        sample_rate: 48_000,
        channels: 2,
    }
}

fn scope(scope: VideoScope) -> ScopeOutput {
    ScopeOutput {
        scope,
        at: RationalTime::new(0, 600).unwrap(),
        width: 640,
        height: 360,
        format: ImageFormat::Png,
    }
}

fn ffmpeg<'a>(
    bundle: &'a veac_codegen::emitter::BackendBundle,
    id: &str,
) -> &'a veac_codegen::emitter::BackendCommand {
    let task = bundle
        .tasks()
        .iter()
        .find(|task| task.deliverable_id.as_str() == id)
        .unwrap();
    let BackendAction::Ffmpeg(command) = &task.action else {
        panic!("expected FFmpeg task")
    };
    command
}

fn pair(arguments: &[String], name: &str, value: &str) -> bool {
    arguments.windows(2).any(|pair| pair == [name, value])
}
