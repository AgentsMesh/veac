use std::{collections::BTreeSet, fs, path::PathBuf};

use serde_json::Value;
use veac_lang::authoring::{
    lower_document, parse, AudioCodec, AudioStemFormat, CaptionSidecarFormat, Document,
    ImageFormat, OutputDecl, OutputEncoding, OutputFormat, ResourceKind, ResourceLocator,
    VideoCodec, VideoScope,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load_fixture(name: &str) -> Document {
    let path = root().join("examples/workflow-evidence").join(name);
    let source = fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    let document = parse(&source).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    let envelope = lower_document(&document).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    veac_ir::validate(&envelope).unwrap_or_else(|error| panic!("{path:?}: {error}"));
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
    match &output.encoding {
        OutputEncoding::Video(value) => {
            ids.insert(container_id(value.container).to_owned());
            ids.insert(video_codec_id(value.video.codec).to_owned());
            if let Some(audio) = &value.audio {
                ids.insert(audio_codec_id(audio.codec).to_owned());
            }
        }
        OutputEncoding::AudioStem(value) => {
            let id = match value.format {
                AudioStemFormat::Wav => "delivery.audio-stem-format.wav",
                AudioStemFormat::Flac => "delivery.audio-stem-format.flac",
            };
            ids.insert(id.to_owned());
        }
        OutputEncoding::ImageSequence(value) => {
            let id = match value.format {
                ImageFormat::Png => "delivery.image-format.png",
                ImageFormat::Jpeg => "delivery.image-format.jpeg",
                ImageFormat::Tiff => "delivery.image-format.tiff",
                ImageFormat::Exr => "delivery.image-format.exr",
            };
            ids.insert(id.to_owned());
        }
        OutputEncoding::CaptionSidecar(value) => {
            let id = match value.format {
                CaptionSidecarFormat::Srt => "delivery.caption-format.srt",
                CaptionSidecarFormat::WebVtt => "delivery.caption-format.web-vtt",
                CaptionSidecarFormat::Ass => "delivery.caption-format.ass",
            };
            ids.insert(id.to_owned());
        }
        OutputEncoding::Scope(value) => {
            let id = match value.scope {
                VideoScope::Waveform => "delivery.scope.waveform",
                VideoScope::Vectorscope => "delivery.scope.vectorscope",
                VideoScope::Histogram => "delivery.scope.histogram",
            };
            ids.insert(id.to_owned());
        }
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
