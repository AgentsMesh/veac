use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

pub(crate) use assert_cmd::Command;
pub(crate) use predicates::prelude::*;
pub(crate) use tempfile::{tempdir, TempDir};

pub(crate) const GENERATED_SOURCE: &str = r#"
project cli-e2e {
  settings {
    timebase 1/600;
    canvas 32px by 24px;
    frame-rate 10fps;
    sample-rate 48000hz;
  }
  entry sequence main;
  sequence main {
    layer visual base {
      item background {
        source generated solid { color #ff0000ff; }
        record { at 0s; duration 200ms; }
      }
    }
  }
  delivery main {
    sequence main;
    raster { canvas 32px by 24px; frame-rate 10fps; captions discard; }
    artifact video main {
      target file "render.mp4";
      mux mp4 {
        layout fast-start;
        video h264 { pixel-format yuv420p; alpha opaque; color-space source; rate-control crf { value 23; } gop automatic; b-frames automatic; profile automatic; level automatic; }
        audio none; passes single; accelerator auto;
      }
    }
  }
}
"#;

pub(crate) const MEDIA_SOURCE: &str = r#"
project media-e2e {
  settings {
    timebase 1/600;
    canvas 32px by 24px;
    frame-rate 10fps;
    sample-rate 48000hz;
  }
  entry sequence main;
  resource video footage {
    locator local { path "clip.mp4"; }
    streams { video auto; audio disabled; }
  }
  sequence main {
    layer video base {
      item footage-1 {
        source media resource footage;
        record { at 0s; duration 200ms; }
        mapping linear { from 0s; to 200ms; }
      }
    }
  }
  delivery main {
    sequence main;
    raster { canvas 32px by 24px; frame-rate 10fps; captions discard; }
    artifact video main {
      target file "media-render.mp4";
      mux mp4 {
        layout fast-start;
        video h264 { pixel-format yuv420p; alpha opaque; color-space source; rate-control crf { value 23; } gop automatic; b-frames automatic; profile automatic; level automatic; }
        audio none; passes single; accelerator auto;
      }
    }
  }
}
"#;

pub(crate) fn veac() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("veac"))
}

pub(crate) fn source_file(temp: &TempDir, source: &str) -> PathBuf {
    let path = temp.path().join("main.veac");
    std::fs::write(&path, source).unwrap();
    path
}

pub(crate) fn compile_ir(temp: &TempDir, source: &str) -> PathBuf {
    let source = source_file(temp, source);
    let project = temp.path().join("project.json");
    veac()
        .args([
            "compile",
            source.to_str().unwrap(),
            "--emit-ir",
            project.to_str().unwrap(),
        ])
        .assert()
        .success();
    project
}

pub(crate) fn make_video(path: &Path) {
    let status = ProcessCommand::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:s=32x24:d=0.4:r=10",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(path)
        .status()
        .expect("FFmpeg is required for CLI E2E tests");
    assert!(status.success());
}

pub(crate) fn video_dimensions(path: &Path) -> String {
    let output = ProcessCommand::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height",
            "-of",
            "csv=p=0:s=x",
        ])
        .arg(path)
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

#[cfg(unix)]
pub(crate) fn fake_audio_ffprobe(root: &Path, index: u32) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let path = root.join("ffprobe-fixture.sh");
    let json = format!(
        r#"{{"streams":[{{"index":{index},"codec_type":"audio","codec_name":"pcm_s16le","time_base":"1/48000","start_time":"0.000000","duration":"1.000000","sample_rate":"48000","channels":2,"channel_layout":"stereo"}}],"format":{{"format_name":"wav","duration":"1.000000"}}}}"#
    );
    std::fs::write(
        &path,
        format!("#!/bin/sh\nif [ \"$1\" = -version ]; then printf 'ffprobe version cli-e2e-fixture-v1\\n'; exit 0; fi\nprintf '%s' '{json}'\n"),
    )
    .unwrap();
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&path, permissions).unwrap();
    path
}
