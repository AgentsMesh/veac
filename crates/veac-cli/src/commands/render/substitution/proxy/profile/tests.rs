use tempfile::tempdir;
use veac_ir::{
    AudioCodec, AudioMixSource, AudioOutput, AudioStemFormat, AudioStemOutput,
    CaptionSidecarFormat, CaptionSidecarOutput, Deliverable, DeliverableId, DeliverableKind,
    DeliverableTarget,
};

use super::{resolve, ProxyAudioFormat};
use crate::unit_tests::support::{canonical_project, FakeEnvironment, GENERATED_SOURCE};

#[test]
fn two_video_artifacts_share_the_delivery_raster() {
    let mut output = output();
    let mut second = output.deliverables[0].clone();
    second.id = DeliverableId::new("dlv_second").unwrap();
    second.target = DeliverableTarget::File {
        name: "second.mp4".into(),
    };
    output.deliverables.push(second);

    let profile = resolve(&output).unwrap();
    assert_eq!(profile.raster, output.raster);
    assert!(profile.audio.is_none());
}

#[test]
fn video_and_stem_with_the_same_shape_share_one_audio_profile() {
    let mut output = output();
    video_audio(&mut output, audio(AudioCodec::Aac, 48_000, 2));
    output.deliverables.push(stem(48_000, 2));

    assert_eq!(
        resolve(&output).unwrap().audio,
        Some(ProxyAudioFormat {
            sample_rate: 48_000,
            channels: 2,
        })
    );
}

#[test]
fn different_audio_rates_or_channels_are_rejected() {
    for (sample_rate, channels) in [(44_100, 2), (48_000, 1)] {
        let mut output = output();
        video_audio(&mut output, audio(AudioCodec::Aac, 48_000, 2));
        output.deliverables.push(stem(sample_rate, channels));
        let error = resolve(&output).unwrap_err();
        assert!(error.to_string().contains("PROXY_AUDIO_FORMAT_AMBIGUOUS"));
    }
}

#[test]
fn audio_only_and_caption_only_deliveries_have_exact_roles() {
    let mut output = output();
    output.raster = None;
    output.deliverables = vec![stem(48_000, 2)];
    let audio = resolve(&output).unwrap();
    assert!(audio.raster.is_none());
    assert!(audio.audio.is_some());

    output.deliverables = vec![Deliverable {
        id: DeliverableId::new("dlv_caption").unwrap(),
        target: DeliverableTarget::File {
            name: "captions.srt".into(),
        },
        kind: DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
            format: CaptionSidecarFormat::Srt,
            track_ids: vec![],
        }),
    }];
    assert!(resolve(&output).unwrap().is_empty());
}

fn output() -> veac_plan::ResolvedOutput {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    crate::planning::prepare(&project, None, &FakeEnvironment::success())
        .unwrap()
        .plan
        .output
}

fn video_audio(output: &mut veac_plan::ResolvedOutput, audio: AudioOutput) {
    output
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap()
        .audio = Some(audio);
}

fn stem(sample_rate: u32, channels: u8) -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_stem").unwrap(),
        target: DeliverableTarget::File {
            name: "stem.wav".into(),
        },
        kind: DeliverableKind::AudioStem(AudioStemOutput {
            format: AudioStemFormat::Wav,
            audio: audio(AudioCodec::PcmS16Le, sample_rate, channels),
            source: AudioMixSource::Master,
        }),
    }
}

fn audio(codec: AudioCodec, sample_rate: u32, channels: u8) -> AudioOutput {
    AudioOutput {
        codec,
        sample_rate,
        channels,
    }
}
