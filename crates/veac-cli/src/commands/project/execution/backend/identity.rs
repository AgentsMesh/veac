use veac_artifact::{ContentDigest, DigestAlgorithm};
use veac_build::ProjectBackendIdentity;

const PROJECT_BACKEND: &str = env!("VEAC_PROJECT_BACKEND_BUILD_SHA256");
const PROJECT_BUILD: &str = env!("VEAC_PROJECT_BUILD_BUILD_SHA256");
const EVIDENCE_BACKEND: &str = env!("VEAC_EVIDENCE_BACKEND_BUILD_SHA256");

pub(super) fn render(ffmpeg: ContentDigest) -> ProjectBackendIdentity {
    ProjectBackendIdentity::VeacRender {
        project_backend: exact(PROJECT_BACKEND),
        build: exact(PROJECT_BUILD),
        compiler: exact(veac_lang::COMPILER_BUILD_SHA256),
        codegen: veac_codegen::render_implementation_identity().digest,
        runtime: veac_runtime::runtime_backend_identity(),
        ffmpeg,
    }
}

pub(super) fn derivation(ffmpeg: ContentDigest, ffprobe: ContentDigest) -> ProjectBackendIdentity {
    ProjectBackendIdentity::MediaDerivation {
        project_backend: exact(PROJECT_BACKEND),
        build: exact(PROJECT_BUILD),
        codegen: veac_codegen::render_implementation_identity().digest,
        runtime: veac_runtime::runtime_backend_identity(),
        ffmpeg,
        ffprobe,
    }
}

pub(super) fn evidence(ffmpeg: ContentDigest, ffprobe: ContentDigest) -> ProjectBackendIdentity {
    ProjectBackendIdentity::Evidence {
        project_backend: exact(PROJECT_BACKEND),
        build: exact(PROJECT_BUILD),
        compiler: exact(veac_lang::COMPILER_BUILD_SHA256),
        evidence: exact(EVIDENCE_BACKEND),
        runtime: veac_runtime::runtime_backend_identity(),
        ffmpeg,
        ffprobe,
    }
}

fn exact(value: &str) -> ContentDigest {
    ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: value.to_owned(),
    }
}

#[cfg(test)]
#[path = "identity/tests.rs"]
mod tests;
