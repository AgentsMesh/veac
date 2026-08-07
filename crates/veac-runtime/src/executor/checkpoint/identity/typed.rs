use veac_artifact::{
    ArtifactParameters, ProducedArtifactParameters, RenderCheckpointParameters,
    RenderOutputParameters, RenderProduct, RenderTaskPhase,
};
use veac_codegen::emitter::{BackendPhase, BackendProduct};

pub(super) fn output(product: BackendProduct, value: RenderOutputParameters) -> ArtifactParameters {
    match product {
        BackendProduct::VideoMaster => {
            ArtifactParameters::VideoMaster(ProducedArtifactParameters::Render(value))
        }
        BackendProduct::RenderPassLog => {
            ArtifactParameters::RenderCheckpoint(RenderCheckpointParameters::Output(value))
        }
        BackendProduct::ImageSequence => ArtifactParameters::ImageSequenceFrame(value),
        BackendProduct::CaptionSidecar => ArtifactParameters::CaptionSidecar(value),
        BackendProduct::AudioStem => {
            ArtifactParameters::AudioStem(ProducedArtifactParameters::Render(value))
        }
        BackendProduct::AudioFile => ArtifactParameters::AudioFile(value),
        BackendProduct::AnimatedImage => ArtifactParameters::AnimatedImage(value),
        BackendProduct::StillImage => ArtifactParameters::StillImage(value),
        BackendProduct::HlsVod => ArtifactParameters::AdaptivePackage(value),
        BackendProduct::VideoWaveform => ArtifactParameters::VideoWaveform(value),
        BackendProduct::Vectorscope => ArtifactParameters::Vectorscope(value),
        BackendProduct::Histogram => ArtifactParameters::Histogram(value),
    }
}

pub(super) fn phase(value: BackendPhase) -> RenderTaskPhase {
    match value {
        BackendPhase::Single => RenderTaskPhase::Single,
        BackendPhase::FirstPass => RenderTaskPhase::FirstPass,
        BackendPhase::SecondPass => RenderTaskPhase::SecondPass,
    }
}

pub(super) fn product(value: BackendProduct) -> RenderProduct {
    match value {
        BackendProduct::VideoMaster => RenderProduct::VideoMaster,
        BackendProduct::RenderPassLog => RenderProduct::RenderPassLog,
        BackendProduct::ImageSequence => RenderProduct::ImageSequence,
        BackendProduct::CaptionSidecar => RenderProduct::CaptionSidecar,
        BackendProduct::AudioStem => RenderProduct::AudioStem,
        BackendProduct::AudioFile => RenderProduct::AudioFile,
        BackendProduct::AnimatedImage => RenderProduct::AnimatedImage,
        BackendProduct::StillImage => RenderProduct::StillImage,
        BackendProduct::HlsVod => RenderProduct::HlsVod,
        BackendProduct::VideoWaveform => RenderProduct::VideoWaveform,
        BackendProduct::Vectorscope => RenderProduct::Vectorscope,
        BackendProduct::Histogram => RenderProduct::Histogram,
    }
}
