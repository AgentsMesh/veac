use tempfile::tempdir;
use veac_ir::{
    AudioCodec, AudioMixSource, AudioOutput, AudioStemFormat, AudioStemOutput,
    CaptionSidecarFormat, CaptionSidecarOutput, DeliverableKind, ImageFormat, ImageSequenceOutput,
    RationalTime, ScopeOutput, VideoScope,
};

use super::support::{canonical_project, FakeEnvironment, GENERATED_SOURCE};

#[test]
fn auxiliary_deliverables_enforce_their_authored_file_contracts() {
    let cases = [
        (
            "captions.SRT",
            DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
                format: CaptionSidecarFormat::Srt,
                track_ids: vec![],
            }),
        ),
        (
            "captions.vtt",
            DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
                format: CaptionSidecarFormat::WebVtt,
                track_ids: vec![],
            }),
        ),
        (
            "captions.ASS",
            DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
                format: CaptionSidecarFormat::Ass,
                track_ids: vec![],
            }),
        ),
        (
            "mix.WAV",
            DeliverableKind::AudioStem(audio_stem(AudioStemFormat::Wav)),
        ),
        (
            "mix.flac",
            DeliverableKind::AudioStem(audio_stem(AudioStemFormat::Flac)),
        ),
        (
            "scope.JPG",
            DeliverableKind::Scope(ScopeOutput {
                scope: VideoScope::Waveform,
                at: RationalTime::zero(600).unwrap(),
                width: 640,
                height: 360,
                format: ImageFormat::Jpeg,
            }),
        ),
        (
            "frame-%d.png",
            DeliverableKind::ImageSequence(ImageSequenceOutput {
                format: ImageFormat::Png,
                start_number: 1,
            }),
        ),
        (
            "frame-%d.tiff",
            DeliverableKind::ImageSequence(ImageSequenceOutput {
                format: ImageFormat::Tiff,
                start_number: 1,
            }),
        ),
        (
            "frame-%d.EXR",
            DeliverableKind::ImageSequence(ImageSequenceOutput {
                format: ImageFormat::Exr,
                start_number: 1,
            }),
        ),
    ];
    for (name, kind) in cases {
        assert_bound(name, kind);
    }
}

#[test]
fn mismatched_auxiliary_extension_is_rejected_before_render() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let mut prepared = crate::planning::prepare_with_material_root(
        &project,
        None,
        None,
        &FakeEnvironment::success(),
    )
    .unwrap();
    let deliverable = &mut prepared.plan.output.deliverables[0];
    deliverable.target = veac_ir::DeliverableTarget::File {
        name: "captions.json".into(),
    };
    deliverable.kind = DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
        format: CaptionSidecarFormat::Srt,
        track_ids: vec![],
    });
    let error = crate::output::bind_render_outputs(&mut prepared, None).unwrap_err();
    assert!(error.to_string().contains("OUTPUT_FORMAT_MISMATCH"));
}

fn assert_bound(name: &str, kind: DeliverableKind) {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let mut prepared = crate::planning::prepare_with_material_root(
        &project,
        None,
        None,
        &FakeEnvironment::success(),
    )
    .unwrap();
    let deliverable = &mut prepared.plan.output.deliverables[0];
    deliverable.target = match kind {
        DeliverableKind::ImageSequence(_) => veac_ir::DeliverableTarget::ImageSequence {
            pattern: name.into(),
        },
        _ => veac_ir::DeliverableTarget::File { name: name.into() },
    };
    deliverable.kind = kind;
    let paths = crate::output::bind_render_outputs(&mut prepared, None).unwrap();
    assert_eq!(paths[0].file_name().unwrap(), name);
}

fn audio_stem(format: AudioStemFormat) -> AudioStemOutput {
    AudioStemOutput {
        format,
        audio: AudioOutput {
            codec: AudioCodec::PcmS16Le,
            sample_rate: 48_000,
            channels: 2,
        },
        source: AudioMixSource::Master,
    }
}
