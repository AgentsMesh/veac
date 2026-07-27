use std::path::PathBuf;

use veac_artifact::ContentDigest;
use veac_plan::canonical::{DeliverableId, MediaIdentity};

mod arguments;
mod filter;
mod requirement;

pub use filter::{BackendFilterBinding, BackendFilterContract, BackendFilterEscape};
pub use requirement::{BackendCapabilityKind, BackendRequirement};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendInput {
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendCommand {
    pub inputs: Vec<BackendInput>,
    pub filter_graph: Option<String>,
    pub filter_contract: Option<BackendFilterContract>,
    pub maps: Vec<String>,
    pub output_args: Vec<String>,
    pub output_path: PathBuf,
}

/// An executable backend bundle issued only by [`super::emit_all`].
///
/// Bundle internals are intentionally read-only outside codegen. This keeps callers from turning
/// arbitrary FFmpeg arguments into an executable bundle while still allowing inspection, logging,
/// and runtime validation.
///
/// ```compile_fail
/// use veac_artifact::ContentDigest;
/// use veac_codegen::emitter::BackendBundle;
///
/// let bundle = BackendBundle {
///     plan_identity: ContentDigest::sha256(b"forged-plan"),
///     substitution_proof: ContentDigest::sha256(b"forged-substitution"),
///     protected_resources: vec![],
///     requirements: vec![],
///     tasks: vec![],
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendBundle {
    plan_identity: ContentDigest,
    substitution_proof: ContentDigest,
    protected_resources: Vec<BackendResource>,
    requirements: Vec<BackendRequirement>,
    tasks: Vec<BackendTask>,
}

impl BackendBundle {
    pub(super) fn new(
        plan_identity: ContentDigest,
        substitution_proof: ContentDigest,
        protected_resources: Vec<BackendResource>,
        requirements: Vec<BackendRequirement>,
        tasks: Vec<BackendTask>,
    ) -> Self {
        Self {
            plan_identity,
            substitution_proof,
            protected_resources,
            requirements,
            tasks,
        }
    }

    pub fn plan_identity(&self) -> &ContentDigest {
        &self.plan_identity
    }

    pub fn substitution_proof(&self) -> &ContentDigest {
        &self.substitution_proof
    }

    pub fn protected_resources(&self) -> &[BackendResource] {
        &self.protected_resources
    }

    pub fn requirements(&self) -> &[BackendRequirement] {
        &self.requirements
    }

    pub fn tasks(&self) -> &[BackendTask] {
        &self.tasks
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendResource {
    pub path: PathBuf,
    pub expected_identity: MediaIdentity,
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
pub enum BackendOutput {
    File(PathBuf),
    Files { paths: Vec<PathBuf> },
    ImageSequence { pattern: PathBuf },
}
