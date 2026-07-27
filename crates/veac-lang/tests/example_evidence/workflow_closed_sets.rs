use std::{collections::BTreeSet, fs, path::PathBuf};

use serde_json::Value;
use veac_lang::authoring::{
    lower, parse, AudioCodec, AudioStemFormat, CaptionSidecarFormat, ContainerFormat, Document,
    ImageFormat, OutputDecl, ResourceKind, ResourceLocator, VideoCodec, VideoScope,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load_fixture(name: &str) -> Document {
    let path = root().join("examples/workflow-evidence").join(name);
    let source = fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    let document = parse(&source).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    let envelope = lower(&document).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    envelope
        .validate()
        .unwrap_or_else(|error| panic!("{path:?}: {error}"));
    document
}

fn catalog_ids(name: &str, keep: impl Fn(&str) -> bool) -> BTreeSet<String> {
    let path = root().join("examples/catalog/mechanisms").join(name);
    let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    value["mechanisms"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["id"].as_str())
        .filter(|id| keep(id))
        .map(str::to_owned)
        .collect()
}

#[test]
fn resource_workflow_fixture_matches_catalog() {
    let document = load_fixture("resources-closed-sets.veac");
    let mut actual = BTreeSet::new();
    for resource in &document.project.resources {
        if resource.kind == ResourceKind::Font {
            actual.insert("resource.kind.font".to_owned());
        }
        if matches!(resource.locator, ResourceLocator::Remote { .. }) {
            actual.insert("resource.locator.remote".to_owned());
        }
    }
    let expected = catalog_ids("generators.json", |id| id.starts_with("resource."));
    assert_eq!(actual, expected);
}

#[test]
fn delivery_workflow_fixture_matches_catalog() {
    let document = load_fixture("delivery-closed-sets.veac");
    let mut actual = BTreeSet::new();
    for output in &document.project.outputs {
        output_ids(output, &mut actual);
    }
    let expected = catalog_ids("delivery-workflows.json", |id| {
        [
            "delivery.container.",
            "delivery.video-codec.",
            "delivery.audio-codec.",
            "delivery.image-format.",
            "delivery.caption-format.",
            "delivery.audio-stem-format.",
            "delivery.scope.",
        ]
        .iter()
        .any(|prefix| id.starts_with(prefix))
            && id != "delivery.audio-codec.mp3"
    });
    assert_eq!(actual, expected);
}

fn output_ids(output: &OutputDecl, ids: &mut BTreeSet<String>) {
    match output {
        OutputDecl::Video(value) => {
            ids.insert(container_id(value.encoding.format).to_owned());
            ids.insert(video_codec_id(value.encoding.video.codec).to_owned());
            if let Some(audio) = &value.encoding.audio {
                ids.insert(audio_codec_id(audio.codec).to_owned());
            }
        }
        OutputDecl::AudioStem(value) => {
            let id = match value.encoding.format {
                AudioStemFormat::Wav => "delivery.audio-stem-format.wav",
                AudioStemFormat::Flac => "delivery.audio-stem-format.flac",
            };
            ids.insert(id.to_owned());
        }
        OutputDecl::ImageSequence(value) => {
            let id = match value.encoding.format {
                ImageFormat::Png => "delivery.image-format.png",
                ImageFormat::Jpeg => "delivery.image-format.jpeg",
                ImageFormat::Tiff => "delivery.image-format.tiff",
                ImageFormat::Exr => "delivery.image-format.exr",
            };
            ids.insert(id.to_owned());
        }
        OutputDecl::CaptionSidecar(value) => {
            let id = match value.encoding.format {
                CaptionSidecarFormat::Srt => "delivery.caption-format.srt",
                CaptionSidecarFormat::WebVtt => "delivery.caption-format.web-vtt",
                CaptionSidecarFormat::Ass => "delivery.caption-format.ass",
            };
            ids.insert(id.to_owned());
        }
        OutputDecl::Scope(value) => {
            let id = match value.encoding.scope {
                VideoScope::Waveform => "delivery.scope.waveform",
                VideoScope::Vectorscope => "delivery.scope.vectorscope",
                VideoScope::Histogram => "delivery.scope.histogram",
            };
            ids.insert(id.to_owned());
        }
    }
}

fn container_id(value: ContainerFormat) -> &'static str {
    match value {
        ContainerFormat::Mp4 => "delivery.container.mp4",
        ContainerFormat::Mov => "delivery.container.mov",
        ContainerFormat::Mkv => "delivery.container.mkv",
        ContainerFormat::WebM => "delivery.container.webm",
        ContainerFormat::Mxf => "delivery.container.mxf",
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
