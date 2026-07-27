use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;
use veac_artifact::ArtifactStore;
use veac_codegen::emitter::{BackendAction, BackendCapabilityKind, BackendRequirement};
use veac_ir::StreamChoice;

use super::support::*;

#[test]
fn codec_audio_boundaries_execute_through_real_ffmpeg() {
    let temp = tempdir().unwrap();
    let tone = tone_fixture(temp.path(), "source", 440);
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_tone",
        MaterialKind::Audio,
        StreamChoice::Disabled,
        StreamChoice::Auto,
    ));
    let mut audio = media_clip("itm_tone", "med_tone", 0, 1_000);
    audio.audio = Some(audio_properties(0.25));
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_picture",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_picture", color(20, 40, 80), 0, 1_000)],
        ),
        track("trk_audio", TrackKind::Audio, 1, vec![audio]),
    ]);
    canonical.project.render_configs[0].deliverables = vec![
        video(
            "dlv_aac",
            "aac.mp4",
            OutputFormat::Mp4,
            VideoCodec::H264,
            AudioCodec::Aac,
            7_350,
        ),
        stem(
            "dlv_flac",
            "high.flac",
            AudioStemFormat::Flac,
            AudioCodec::Flac,
            192_000,
        ),
        video(
            "dlv_opus",
            "opus.webm",
            OutputFormat::Webm,
            VideoCodec::Vp9,
            AudioCodec::Opus,
            8_000,
        ),
        stem(
            "dlv_pcm",
            "high.wav",
            AudioStemFormat::Wav,
            AudioCodec::PcmS24Le,
            384_000,
        ),
    ];
    let assets = BTreeMap::from([("med_tone".to_owned(), tone)]);
    let delivery = prepare_delivery(canonical, &assets, temp.path());
    for encoder in ["aac", "flac", "libopus", "pcm_s24le"] {
        assert!(delivery.bundle.requirements().iter().any(|value| matches!(
            value,
            BackendRequirement::Encoder { name, .. } if name == encoder
        )));
    }
    for (kind, name) in [
        (BackendCapabilityKind::Decoder, "aac"),
        (BackendCapabilityKind::Demuxer, "mov"),
        (BackendCapabilityKind::Filter, "atrim"),
        (BackendCapabilityKind::Muxer, "mp4"),
        (BackendCapabilityKind::Muxer, "flac"),
        (BackendCapabilityKind::Muxer, "webm"),
        (BackendCapabilityKind::Muxer, "wav"),
    ] {
        assert!(delivery
            .bundle
            .requirements()
            .iter()
            .any(|value| value.kind() == kind && value.name() == name));
    }
    delivery.execute(&ArtifactStore::new(temp.path().join("store")));
    assert_stream(delivery.path("dlv_aac"), "aac", 7_350);
    assert_stream(delivery.path("dlv_flac"), "flac", 192_000);
    assert_stream(delivery.path("dlv_pcm"), "pcm_s24le", 384_000);
    assert_eq!(stream(delivery.path("dlv_opus"))["codec_name"], "opus");
    assert!(output_args(&delivery, "dlv_opus")
        .windows(2)
        .any(|pair| pair == ["-ar", "8000"]));
}

fn video(
    id: &str,
    file: &str,
    container: OutputFormat,
    video: VideoCodec,
    audio: AudioCodec,
    sample_rate: u32,
) -> Deliverable {
    let settings = VideoDeliverable {
        container,
        video: VideoOutput {
            codec: video,
            ..VideoOutput::default()
        },
        audio: Some(AudioOutput {
            codec: audio,
            sample_rate,
            channels: 1,
        }),
        ..VideoDeliverable::default()
    };
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        file_name: file.to_owned(),
        kind: DeliverableKind::Video(settings),
    }
}

fn stem(
    id: &str,
    file: &str,
    format: AudioStemFormat,
    codec: AudioCodec,
    sample_rate: u32,
) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        file_name: file.to_owned(),
        kind: DeliverableKind::AudioStem(AudioStemOutput {
            format,
            audio: AudioOutput {
                codec,
                sample_rate,
                channels: 1,
            },
            source: AudioStemSource::Master,
        }),
    }
}

fn assert_stream(path: &Path, codec: &str, sample_rate: u32) {
    let stream = stream(path);
    assert_eq!(stream["codec_name"], codec);
    assert_eq!(stream["sample_rate"], sample_rate.to_string());
    assert_eq!(stream["channels"], 1);
}

fn stream(path: &Path) -> Value {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=codec_name,sample_rate,channels",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["streams"][0].clone()
}

fn output_args<'a>(delivery: &'a PreparedDelivery, id: &str) -> &'a [String] {
    let task = delivery
        .bundle
        .tasks()
        .iter()
        .find(|task| task.deliverable_id.as_str() == id)
        .unwrap();
    let BackendAction::Ffmpeg(command) = &task.action else {
        panic!()
    };
    &command.output_args
}
