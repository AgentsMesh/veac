use std::ffi::OsString;
use std::path::Path;

use rustix::fd::OwnedFd;
use rustix::fs::{openat, Mode, OFlags};

use super::{io_error, write_error};
use crate::error::{CliError, CliResult};

pub(super) fn unique(directory: &OwnedFd, label: &Path) -> CliResult<(OsString, OwnedFd)> {
    for _ in 0..128 {
        let name = cli_try!(random_name(), |error| io_error(label, "name stage", error));
        let flags =
            OFlags::CREATE | OFlags::EXCL | OFlags::RDWR | OFlags::NOFOLLOW | OFlags::CLOEXEC;
        match openat(directory, &name, flags, Mode::RUSR | Mode::WUSR) {
            Ok(file) => return Ok((name, file)),
            Err(rustix::io::Errno::EXIST) => continue,
            Err(error) => return Err(write_error(label, "create stage", error)),
        }
    }
    Err(CliError::new(
        "WRITE_FAILED",
        "cannot allocate a unique source stage",
    ))
}

fn random_name() -> std::io::Result<OsString> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(std::io::Error::other)?;
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    Ok(format!(".veac-source-stage-{encoded}").into())
}
