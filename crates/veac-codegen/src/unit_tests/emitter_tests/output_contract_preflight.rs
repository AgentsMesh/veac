use veac_codegen::emitter::emit_all;
use veac_plan::canonical::*;

use super::support::{bindings, fixture, resolved, text_fixture};

#[test]
fn output_names_extensions_and_patterns_fail_closed() {
    let mut unsafe_name = resolved(&fixture());
    let local = bindings(&unsafe_name);
    unsafe_name.output.deliverables[0].file_name = "../escape.mp4".into();
    assert_code_with(&unsafe_name, &local, "PLAN_DELIVERABLE_NAME_INVALID");

    let mut extension = resolved(&fixture());
    let local = bindings(&extension);
    extension.output.deliverables[0].file_name = "master.mov".into();
    assert_code_with(&extension, &local, "PLAN_VIDEO_FILE_FORMAT_INVALID");

    let mut image = resolved(&fixture());
    let local = bindings(&image);
    image.output.deliverables[0].file_name = "frame.png".into();
    image.output.deliverables[0].kind = DeliverableKind::ImageSequence(ImageSequenceOutput {
        format: ImageFormat::Png,
        start_number: 0,
    });
    assert_code_with(&image, &local, "PLAN_DELIVERABLE_NAME_INVALID");
}

#[test]
fn padded_image_sequence_pattern_passes_preflight() {
    let mut image = resolved(&fixture());
    image.output.deliverables[0].file_name = "frame-%04d.png".into();
    image.output.deliverables[0].kind = DeliverableKind::ImageSequence(ImageSequenceOutput {
        format: ImageFormat::Png,
        start_number: 0,
    });
    let local = bindings(&image);
    emit_all(&image, &local).unwrap();
}

#[test]
fn overlapping_patterns_and_invalid_aux_sources_fail_closed() {
    let mut collision = resolved(&fixture());
    collision.output.deliverables = vec![
        deliverable(
            "dlv_frames",
            "frame-%d.png",
            DeliverableKind::ImageSequence(ImageSequenceOutput {
                format: ImageFormat::Png,
                start_number: 0,
            }),
        ),
        deliverable(
            "dlv_still",
            "frame-1.png",
            DeliverableKind::Scope(ScopeOutput {
                scope: VideoScope::Histogram,
                at: RationalTime::zero(600).unwrap(),
                width: 320,
                height: 180,
                format: ImageFormat::Png,
            }),
        ),
    ];
    assert_code(&collision, "PLAN_DELIVERABLE_COLLISION");

    let mut caption = resolved(&text_fixture(true));
    caption.output.deliverables[0].file_name = "captions.vtt".into();
    caption.output.deliverables[0].kind = DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
        format: CaptionSidecarFormat::WebVtt,
        track_ids: vec![TrackId::new("trk_video").unwrap()],
    });
    assert_code(&caption, "PLAN_CAPTION_SIDECAR_INVALID");

    let mut stem = resolved(&fixture());
    stem.output.deliverables[0].file_name = "missing.wav".into();
    stem.output.deliverables[0].kind = DeliverableKind::AudioStem(AudioStemOutput {
        format: AudioStemFormat::Wav,
        audio: AudioOutput {
            codec: AudioCodec::PcmS16Le,
            sample_rate: 48_000,
            channels: 2,
        },
        source: AudioStemSource::Track {
            track_id: TrackId::new("trk_missing").unwrap(),
        },
    });
    assert_code(&stem, "PLAN_AUDIO_STEM_INVALID");
}

fn deliverable(id: &str, file_name: &str, kind: DeliverableKind) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        file_name: file_name.into(),
        kind,
    }
}

fn assert_code(plan: &veac_plan::ResolvedRenderPlan, expected: &str) {
    assert_code_with(plan, &bindings(plan), expected);
}

fn assert_code_with(
    plan: &veac_plan::ResolvedRenderPlan,
    local: &veac_artifact::ExecutionBindings,
    expected: &str,
) {
    let error = emit_all(plan, local).unwrap_err();
    assert!(
        error
            .diagnostics()
            .iter()
            .any(|value| value.code == expected),
        "missing {expected}: {error}"
    );
}
