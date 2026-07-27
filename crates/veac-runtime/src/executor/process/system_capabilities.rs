use std::collections::BTreeSet;
use std::time::Instant;

use veac_artifact::ContentDigest;
use veac_codegen::emitter::BackendCapabilityKind;

use super::{
    append_field, capability, ensure_success, FfmpegEnvironment, FfmpegFingerprint,
    FfmpegInvocation, SystemFfmpeg,
};
use crate::RuntimeError;

impl FfmpegEnvironment for SystemFfmpeg {
    fn fingerprint(&self) -> Result<FfmpegFingerprint, RuntimeError> {
        self.fingerprint_until(super::hard_deadline())
    }

    fn encoders(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.table("-encoders", "encoder", capability::parse_encoders)
    }

    fn decoders(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.table("-decoders", "decoder", capability::parse_decoders)
    }

    fn muxers(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.table("-muxers", "muxer", capability::parse_muxers)
    }

    fn demuxers(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.table("-demuxers", "demuxer", capability::parse_demuxers)
    }

    fn filters(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.table("-filters", "filter", capability::parse_filters)
    }

    fn hardware_backends(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.table("-hwaccels", "hardware backend", capability::parse_names)
    }

    fn hardware_devices(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.hardware_devices_until(super::hard_deadline())
    }

    fn fingerprint_until(&self, deadline: Instant) -> Result<FfmpegFingerprint, RuntimeError> {
        SystemFfmpeg::fingerprint_until(self, deadline)
    }

    fn capability_until(
        &self,
        kind: BackendCapabilityKind,
        deadline: Instant,
    ) -> Result<BTreeSet<String>, RuntimeError> {
        match kind {
            BackendCapabilityKind::Encoder => {
                self.table_until("-encoders", "encoder", capability::parse_encoders, deadline)
            }
            BackendCapabilityKind::Decoder => {
                self.table_until("-decoders", "decoder", capability::parse_decoders, deadline)
            }
            BackendCapabilityKind::Muxer => {
                self.table_until("-muxers", "muxer", capability::parse_muxers, deadline)
            }
            BackendCapabilityKind::Demuxer => {
                self.table_until("-demuxers", "demuxer", capability::parse_demuxers, deadline)
            }
            BackendCapabilityKind::Filter => {
                self.table_until("-filters", "filter", capability::parse_filters, deadline)
            }
            BackendCapabilityKind::HardwareBackend => self.table_until(
                "-hwaccels",
                "hardware backend",
                capability::parse_names,
                deadline,
            ),
            BackendCapabilityKind::HardwareDevice => self.hardware_devices_until(deadline),
        }
    }

    fn execute(&self, invocation: FfmpegInvocation<'_>) -> Result<(), RuntimeError> {
        let output = self.run_until(
            invocation.arguments(),
            invocation.output_root,
            invocation.deadline,
        )?;
        ensure_success(&output, "FFmpeg render")
    }
}

impl SystemFfmpeg {
    fn hardware_devices_until(&self, deadline: Instant) -> Result<BTreeSet<String>, RuntimeError> {
        let output = self.run_until(
            &[
                "-hide_banner".to_owned(),
                "-init_hw_device".to_owned(),
                "list".to_owned(),
            ],
            None,
            deadline,
        )?;
        ensure_success(&output, "FFmpeg hardware device probe")?;
        Ok(capability::parse_names(&String::from_utf8_lossy(
            &output.stderr,
        )))
    }
    pub(crate) fn fingerprint_until(
        &self,
        deadline: Instant,
    ) -> Result<FfmpegFingerprint, RuntimeError> {
        let executable_identity = self.pinned_until(deadline)?.identity().clone();
        let output = self.run_until(&["-version".to_owned()], None, deadline)?;
        ensure_success(&output, "FFmpeg version probe")?;
        let version = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| RuntimeError::new("FFmpeg returned an empty version fingerprint"))?
            .to_owned();
        let mut fingerprint = b"veac.ffmpeg-fingerprint.v2".to_vec();
        append_field(&mut fingerprint, executable_identity.digest.as_bytes());
        append_field(&mut fingerprint, &output.stdout);
        append_field(&mut fingerprint, &output.stderr);
        Ok(FfmpegFingerprint {
            version,
            configuration: ContentDigest::sha256(fingerprint),
        })
    }

    fn table(
        &self,
        option: &str,
        name: &str,
        parse: fn(&str) -> BTreeSet<String>,
    ) -> Result<BTreeSet<String>, RuntimeError> {
        self.table_until(option, name, parse, super::hard_deadline())
    }

    fn table_until(
        &self,
        option: &str,
        name: &str,
        parse: fn(&str) -> BTreeSet<String>,
        deadline: Instant,
    ) -> Result<BTreeSet<String>, RuntimeError> {
        let output = self.run_until(
            &["-hide_banner".to_owned(), option.to_owned()],
            None,
            deadline,
        )?;
        ensure_success(&output, &format!("FFmpeg {name} probe"))?;
        Ok(parse(&String::from_utf8_lossy(&output.stdout)))
    }
}
