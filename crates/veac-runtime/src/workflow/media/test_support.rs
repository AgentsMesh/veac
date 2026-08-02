#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::MediaWorkflow;

pub(super) fn workflow(ffmpeg: &Path, root: &Path) -> MediaWorkflow {
    MediaWorkflow::with_tools(ffmpeg, fake_ffprobe(root))
}

pub(super) fn fake_ffprobe(root: &Path) -> PathBuf {
    executable(
        root.join("fake-ffprobe.sh"),
        r#"#!/bin/sh
if [ "$1" = "-version" ]; then
  printf 'ffprobe version fake-media-v1\n'
  exit 0
fi
printf '%s' '{"format":{"format_name":"wav","duration":"1.000000"},"streams":[{"index":0,"codec_type":"audio","codec_name":"pcm_s16le","time_base":"1/48000","start_time":"0.000000","duration":"1.000000","sample_rate":"48000","channels":2,"channel_layout":"stereo"}]}'
"#,
    )
}

pub(super) fn executable(path: PathBuf, contents: &str) -> PathBuf {
    fs::write(&path, contents).unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&path, permissions).unwrap();
    path
}
