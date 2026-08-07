use std::{collections::BTreeSet, fs, path::PathBuf};

use serde_json::Value;
use veac_ir::{
    AudioCodec, AudioStemFormat, CaptionSidecarFormat, Deliverable, DeliverableKind, ImageFormat,
    MaterialKind, MaterialSource, OutputFormat, ProjectEnvelope, VideoCodec, VideoScope,
};
use veac_lang::program::build_path;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load_fixture(name: &str) -> ProjectEnvelope {
    let path = root().join("examples/workflow-evidence").join(name);
    build_path(&path)
        .unwrap_or_else(|diagnostics| panic!("{}: {diagnostics:?}", path.display()))
        .envelope()
        .clone()
}

fn catalog_ids(name: &str, keep: impl Fn(&Value) -> bool) -> BTreeSet<String> {
    let path = root().join("examples/catalog/mechanisms").join(name);
    let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    value["mechanisms"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| keep(item))
        .filter_map(|item| item["id"].as_str())
        .map(str::to_owned)
        .collect()
}

#[test]
fn resource_workflow_fixture_matches_catalog() {
    let project = load_fixture("resources-closed-sets.veac");
    let mut actual = BTreeSet::new();
    for resource in &project.project.materials {
        if resource.kind == MaterialKind::Font {
            actual.insert("resource.kind.font".to_owned());
        }
        if matches!(resource.source, MaterialSource::Remote { .. }) {
            actual.insert("resource.locator.remote".to_owned());
        }
    }
    let expected = catalog_ids("generators.json", |item| {
        item["coverage"] == "workflow_evidence"
            && item["id"]
                .as_str()
                .is_some_and(|id| id.starts_with("resource."))
    });
    assert_eq!(actual, expected);
}

#[test]
fn delivery_workflow_fixture_matches_catalog() {
    let project = load_fixture("delivery-closed-sets.veac");
    let mut actual = BTreeSet::new();
    for artifact in project
        .project
        .render_configs
        .iter()
        .flat_map(|delivery| &delivery.deliverables)
    {
        artifact_ids(artifact, &mut actual);
    }
    assert_eq!(actual, catalog_ids("delivery-workflows.json", |_| true));
}

fn artifact_ids(artifact: &Deliverable, ids: &mut BTreeSet<String>) {
    match &artifact.kind {
        DeliverableKind::Video(value) => {
            ids.insert(container_id(value.container).to_owned());
            ids.insert(video_codec_id(value.video.codec).to_owned());
            if let Some(audio) = &value.audio {
                ids.insert(audio_codec_id(audio.codec).to_owned());
            }
        }
        DeliverableKind::AudioStem(value) => {
            ids.insert("delivery.audio-only".to_owned());
            ids.insert(
                match value.format {
                    AudioStemFormat::Wav => "delivery.audio-stem-format.wav",
                    AudioStemFormat::Flac => "delivery.audio-stem-format.flac",
                }
                .to_owned(),
            );
        }
        DeliverableKind::ImageSequence(value) => {
            ids.insert("delivery.image-sequence".to_owned());
            ids.insert(image_format_id(value.format).to_owned());
        }
        DeliverableKind::CaptionSidecar(value) => {
            ids.insert("delivery.sidecar.captions".to_owned());
            ids.insert(
                match value.format {
                    CaptionSidecarFormat::Srt => "delivery.caption-format.srt",
                    CaptionSidecarFormat::WebVtt => "delivery.caption-format.web-vtt",
                    CaptionSidecarFormat::Ass => "delivery.caption-format.ass",
                }
                .to_owned(),
            );
        }
        DeliverableKind::Scope(value) => {
            ids.insert(
                match value.scope {
                    VideoScope::Waveform => "delivery.scope.waveform",
                    VideoScope::Vectorscope => "delivery.scope.vectorscope",
                    VideoScope::Histogram => "delivery.scope.histogram",
                }
                .to_owned(),
            );
        }
        DeliverableKind::AudioFile(_) => {
            ids.extend(["delivery.audio-codec.mp3", "delivery.audio-only"].map(str::to_owned));
        }
        DeliverableKind::AnimatedImage(_) => {
            ids.insert("delivery.gif".to_owned());
        }
        DeliverableKind::StillImage(_) => {
            ids.insert("delivery.single-frame".to_owned());
        }
        DeliverableKind::AdaptivePackage(_) => {
            ids.insert("delivery.multiresolution-hls".to_owned());
        }
    }
}

fn image_format_id(value: ImageFormat) -> &'static str {
    match value {
        ImageFormat::Png => "delivery.image-format.png",
        ImageFormat::Jpeg => "delivery.image-format.jpeg",
        ImageFormat::Tiff => "delivery.image-format.tiff",
        ImageFormat::Exr => "delivery.image-format.exr",
    }
}

fn container_id(value: OutputFormat) -> &'static str {
    match value {
        OutputFormat::Mp4 => "delivery.container.mp4",
        OutputFormat::Mov => "delivery.container.mov",
        OutputFormat::Mkv => "delivery.container.mkv",
        OutputFormat::Webm => "delivery.container.webm",
        OutputFormat::Mxf => "delivery.container.mxf",
    }
}

fn video_codec_id(value: VideoCodec) -> &'static str {
    match value {
        VideoCodec::H264 => "delivery.video-codec.h264",
        VideoCodec::H265 => "delivery.video-codec.h265",
        VideoCodec::Vp9 => "delivery.video-codec.vp9",
        VideoCodec::Av1 => "delivery.video-codec.av1",
        VideoCodec::ProRes => "delivery.video-codec.prores",
        VideoCodec::DnxHr => "delivery.video-codec.dnxhr",
    }
}

fn audio_codec_id(value: AudioCodec) -> &'static str {
    match value {
        AudioCodec::Aac => "delivery.audio-codec.aac",
        AudioCodec::Opus => "delivery.audio-codec.opus",
        AudioCodec::PcmS16Le => "delivery.audio-codec.pcm-s16le",
        AudioCodec::PcmS24Le => "delivery.audio-codec.pcm-s24le",
        AudioCodec::PcmS32Le => "delivery.audio-codec.pcm-s32le",
        AudioCodec::Flac => "delivery.audio-codec.flac",
    }
}
