use std::ffi::OsString;
use std::path::Path;
use std::time::{Duration, Instant};

use veac_artifact::verify_source_bounded_while;
use veac_ir::{MediaProbeSnapshot, StreamIntent};

use super::{parse_ffprobe_json, ProbeError};
use crate::input_policy;

mod process;
mod source;
mod tool;
mod version;

pub use tool::SystemFfprobe;

/// Probe a local file using deterministic automatic video and audio selection.
pub fn probe(path: &Path) -> Result<MediaProbeSnapshot, ProbeError> {
    SystemFfprobe::default().probe(path)
}

/// Probe a local file and resolve streams according to the supplied canonical intent.
pub fn probe_with_intent(
    path: &Path,
    intent: StreamIntent,
) -> Result<MediaProbeSnapshot, ProbeError> {
    SystemFfprobe::default().probe_with_intent(path, intent)
}

impl SystemFfprobe {
    pub fn probe(&self, path: &Path) -> Result<MediaProbeSnapshot, ProbeError> {
        self.probe_with_intent(path, super::auto_stream_intent())
    }

    pub fn probe_with_intent(
        &self,
        path: &Path,
        intent: StreamIntent,
    ) -> Result<MediaProbeSnapshot, ProbeError> {
        self.probe_with_intent_bounded(
            path,
            intent,
            veac_artifact::MAX_MEDIA_DERIVATION_WALL_SECONDS,
        )
    }

    pub fn probe_with_intent_bounded(
        &self,
        path: &Path,
        intent: StreamIntent,
        max_wall_seconds: u64,
    ) -> Result<MediaProbeSnapshot, ProbeError> {
        if max_wall_seconds == 0
            || max_wall_seconds > veac_artifact::MAX_MEDIA_DERIVATION_WALL_SECONDS
        {
            return Err(ProbeError::ResourceLimit {
                operation: "wall-clock policy",
            });
        }
        let deadline = Instant::now() + Duration::from_secs(max_wall_seconds);
        self.probe_with_intent_until(path, intent, deadline)
    }

    pub fn probe_with_intent_until(
        &self,
        path: &Path,
        intent: StreamIntent,
        deadline: Instant,
    ) -> Result<MediaProbeSnapshot, ProbeError> {
        if Instant::now() >= deadline {
            return Err(ProbeError::ResourceLimit {
                operation: "wall-clock deadline",
            });
        }
        probe_with_tool(path, intent, self, deadline)
    }
}

fn probe_with_tool(
    path: &Path,
    intent: StreamIntent,
    tool: &SystemFfprobe,
    deadline: Instant,
) -> Result<MediaProbeSnapshot, ProbeError> {
    if !path.exists() {
        return Err(ProbeError::FileNotFound {
            path: path.to_path_buf(),
        });
    }
    let source = source::ProbeSource::capture(path, deadline)?;
    let observed_identity = source.identity().clone();
    let binary = tool.binary();
    let pinned = match tool.pinned_until(deadline) {
        Ok(pinned) => pinned,
        Err(error) => return Err(version::tool_error(binary, error)),
    };
    let engine = version::read(binary, pinned, deadline)?;
    let executable = match pinned.launch_until(deadline) {
        Ok(executable) => executable,
        Err(error) => return Err(version::tool_error(binary, error)),
    };
    let mut arguments = strings(&[
        "-v",
        "error",
        "-print_format",
        "json",
        "-show_format",
        "-show_streams",
    ]);
    arguments.extend(input_policy::os_arguments());
    arguments.push(OsString::from("-i"));
    arguments.push(source.path().as_os_str().to_owned());
    let output = process::run(
        executable.path(),
        &arguments,
        veac_artifact::MAX_ARTIFACT_METADATA_BYTES,
        deadline,
        "media inspection",
    )?;
    if !output.status.success() {
        return Err(ProbeError::ProcessFailed {
            path: path.to_path_buf(),
            status: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }
    verify_source_bounded_while(
        path,
        Some(&observed_identity),
        veac_artifact::MAX_VERIFIED_SOURCE_BYTES,
        || Instant::now() < deadline,
    )
    .map_err(|error| {
        if error.kind == veac_artifact::ArtifactErrorKind::ResourceLimit {
            ProbeError::ResourceLimit {
                operation: "source verification",
            }
        } else {
            ProbeError::IdentityChanged {
                path: path.to_path_buf(),
            }
        }
    })?;
    parse_ffprobe_json(
        &String::from_utf8_lossy(&output.stdout),
        observed_identity,
        intent,
        &engine,
    )
}

fn strings(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}
