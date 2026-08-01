use std::path::PathBuf;

use veac_plan::canonical::DeliverableId;

use super::BackendFilterContract;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendInput {
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendCommand {
    pub preparations: Vec<BackendPreparation>,
    pub inputs: Vec<BackendInput>,
    pub filter_graph: Option<String>,
    pub filter_contract: Option<BackendFilterContract>,
    pub maps: Vec<String>,
    pub output_args: Vec<String>,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendPreparation {
    pub command: BackendCommand,
    pub outputs: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendPhase {
    Single,
    FirstPass,
    SecondPass,
}

impl BackendPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::FirstPass => "pass_1",
            Self::SecondPass => "pass_2",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendTask {
    pub deliverable_id: DeliverableId,
    pub phase: BackendPhase,
    pub product: BackendProduct,
    pub output: BackendOutput,
    pub action: BackendAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendProduct {
    VideoMaster,
    RenderPassLog,
    ImageSequence,
    CaptionSidecar,
    AudioStem,
    AudioFile,
    AnimatedImage,
    StillImage,
    HlsVod,
    VideoWaveform,
    Vectorscope,
    Histogram,
}

impl BackendProduct {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::VideoMaster => "video_master",
            Self::RenderPassLog => "render_pass_log",
            Self::ImageSequence => "image_sequence",
            Self::CaptionSidecar => "caption_sidecar",
            Self::AudioStem => "audio_stem",
            Self::AudioFile => "audio_file",
            Self::AnimatedImage => "animated_image",
            Self::StillImage => "still_image",
            Self::HlsVod => "hls_vod",
            Self::VideoWaveform => "video_waveform",
            Self::Vectorscope => "vectorscope",
            Self::Histogram => "histogram",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendAction {
    Ffmpeg(BackendCommand),
    WriteFile { path: PathBuf, content: Vec<u8> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendPackagePaths {
    pub playlist_pattern: PathBuf,
    pub segment_pattern: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendOutput {
    File(PathBuf),
    Files {
        paths: Vec<PathBuf>,
    },
    ImageSequence {
        pattern: PathBuf,
    },
    Package {
        root: PathBuf,
        entrypoint: PathBuf,
        paths: BackendPackagePaths,
    },
}
