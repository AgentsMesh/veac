use veac_artifact::ContentDigest;
use veac_codegen::emitter::BackendTask;

use super::process::FfmpegFingerprint;
use crate::RuntimeError;

mod error;
mod fresh;
mod identity;
mod manifest;
mod model;
mod resume;
mod store;

#[cfg(test)]
mod tests;

pub(super) use fresh::FreshCheckpoints;
pub(super) use identity::TaskIdentity;
#[cfg(test)]
pub(super) use resume::invalidate;
pub(super) use resume::resume;
#[cfg(test)]
pub(super) use resume::validate_cached;
pub(super) use store::store;

#[cfg(test)]
pub(super) use super::staging::StagedFile;

pub(super) fn identity(
    task: &BackendTask,
    plan: &ContentDigest,
    resources: &ContentDigest,
    ffmpeg: Option<&FfmpegFingerprint>,
    predecessor: Option<&ContentDigest>,
) -> Result<TaskIdentity, RuntimeError> {
    identity::task(task, plan, resources, ffmpeg, predecessor)
}
