use std::collections::BTreeSet;
use std::path::Path;
use std::time::Instant;

use veac_codegen::emitter::BackendCapabilityKind;

use super::FfmpegFingerprint;
use crate::RuntimeError;

/// A runtime-issued capability for one validated bundle task invocation.
///
/// Only executor staging can construct this value. External environments may inspect it, but safe
/// Rust callers cannot mint one to execute an arbitrary backend command.
///
/// ```compile_fail
/// use veac_runtime::executor::{FfmpegEnvironment, FfmpegInvocation, SystemFfmpeg};
///
/// let arguments = vec!["-version".to_owned()];
/// let invocation = FfmpegInvocation::new(&arguments);
/// FfmpegEnvironment::execute(&SystemFfmpeg::default(), invocation).unwrap();
/// ```
pub struct FfmpegInvocation<'a> {
    arguments: &'a [String],
    pub(super) output_root: Option<&'a Path>,
    pub(super) deadline: Instant,
}

impl<'a> FfmpegInvocation<'a> {
    pub(in crate::executor) fn render(
        arguments: &'a [String],
        output_root: &'a Path,
        deadline: Instant,
    ) -> Self {
        Self {
            arguments,
            output_root: Some(output_root),
            deadline,
        }
    }

    pub fn arguments(&self) -> &'a [String] {
        self.arguments
    }

    pub fn deadline(&self) -> Instant {
        self.deadline
    }
}

pub trait FfmpegEnvironment {
    fn fingerprint(&self) -> Result<FfmpegFingerprint, RuntimeError>;
    fn encoders(&self) -> Result<BTreeSet<String>, RuntimeError>;
    fn muxers(&self) -> Result<BTreeSet<String>, RuntimeError>;
    fn decoders(&self) -> Result<BTreeSet<String>, RuntimeError> {
        Ok(BTreeSet::new())
    }
    fn demuxers(&self) -> Result<BTreeSet<String>, RuntimeError> {
        Ok(BTreeSet::new())
    }
    fn filters(&self) -> Result<BTreeSet<String>, RuntimeError> {
        Ok(BTreeSet::new())
    }
    fn hardware_backends(&self) -> Result<BTreeSet<String>, RuntimeError> {
        Ok(BTreeSet::new())
    }
    fn hardware_devices(&self) -> Result<BTreeSet<String>, RuntimeError> {
        Ok(BTreeSet::new())
    }
    fn execute(&self, invocation: FfmpegInvocation<'_>) -> Result<(), RuntimeError>;

    fn fingerprint_until(&self, deadline: Instant) -> Result<FfmpegFingerprint, RuntimeError> {
        crate::executor::deadline::ensure_setup(deadline)?;
        let value = self.fingerprint()?;
        crate::executor::deadline::ensure_setup(deadline)?;
        Ok(value)
    }

    fn capability_until(
        &self,
        kind: BackendCapabilityKind,
        deadline: Instant,
    ) -> Result<BTreeSet<String>, RuntimeError> {
        crate::executor::deadline::ensure_setup(deadline)?;
        let value = match kind {
            BackendCapabilityKind::Encoder => self.encoders(),
            BackendCapabilityKind::Decoder => self.decoders(),
            BackendCapabilityKind::Muxer => self.muxers(),
            BackendCapabilityKind::Demuxer => self.demuxers(),
            BackendCapabilityKind::Filter => self.filters(),
            BackendCapabilityKind::HardwareBackend => self.hardware_backends(),
            BackendCapabilityKind::HardwareDevice => self.hardware_devices(),
        }?;
        crate::executor::deadline::ensure_setup(deadline)?;
        Ok(value)
    }
}

#[cfg(test)]
#[path = "environment/tests.rs"]
mod tests;
