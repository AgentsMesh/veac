use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Instant;

use tempfile::{Builder, TempDir};
use veac_artifact::{copy_verified_source_bounded_while, ArtifactErrorKind};
use veac_ir::MediaIdentity;

use super::ProbeError;

pub(super) struct ProbeSource {
    _directory: TempDir,
    path: PathBuf,
    identity: MediaIdentity,
    image_demuxer: Option<&'static str>,
}

impl ProbeSource {
    pub(super) fn capture(source: &Path, deadline: Instant) -> Result<Self, ProbeError> {
        let directory = match Builder::new().prefix(".veac-probe-source-").tempdir() {
            Ok(directory) => directory,
            Err(error) => return Err(io_error("create snapshot directory", source, error)),
        };
        let mut path = directory.path().join("media");
        if let Some(extension) = source.extension() {
            path.set_extension(extension);
        }
        let copied = match copy_verified_source_bounded_while(
            source,
            &path,
            None,
            veac_artifact::MAX_VERIFIED_SOURCE_BYTES,
            || Instant::now() < deadline,
        ) {
            Ok(copied) => copied,
            Err(error) if error.kind == ArtifactErrorKind::ResourceLimit => {
                return Err(ProbeError::ResourceLimit {
                    operation: "source snapshot",
                });
            }
            Err(error) => {
                return Err(ProbeError::Io {
                    operation: "snapshot",
                    path: source.to_path_buf(),
                    source: std::io::Error::other(error),
                })
            }
        };
        readonly(&path).map_err(|error| io_error("protect snapshot", source, error))?;
        Ok(Self {
            _directory: directory,
            path,
            identity: copied.identity,
            image_demuxer: static_image_demuxer(source),
        })
    }

    pub(super) fn identity(&self) -> &MediaIdentity {
        &self.identity
    }

    pub(super) fn append_input_arguments(&self, arguments: &mut Vec<OsString>) {
        if let Some(demuxer) = self.image_demuxer {
            arguments.extend([OsString::from("-f"), OsString::from(demuxer)]);
        }
        arguments.push(OsString::from("-i"));
        arguments.push(self.path.as_os_str().to_owned());
    }
}

fn static_image_demuxer(path: &Path) -> Option<&'static str> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    Some(match extension.as_str() {
        "bmp" => "bmp_pipe",
        "dds" => "dds_pipe",
        "dpx" => "dpx_pipe",
        "exr" => "exr_pipe",
        "jfif" | "jpe" | "jpeg" | "jpg" => "jpeg_pipe",
        "jls" => "jpegls_pipe",
        "jxl" => "jpegxl_pipe",
        "png" => "png_pipe",
        "psd" => "psd_pipe",
        "tif" | "tiff" => "tiff_pipe",
        "webp" => "webp_pipe",
        _ => return None,
    })
}

fn readonly(path: &Path) -> std::io::Result<()> {
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(path, permissions)
}

fn io_error(operation: &'static str, path: &Path, source: std::io::Error) -> ProbeError {
    ProbeError::Io {
        operation,
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests;
