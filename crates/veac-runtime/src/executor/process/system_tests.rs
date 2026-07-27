#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant};

use super::*;
use veac_codegen::emitter::BackendCapabilityKind;

#[test]
fn version_then_original_replacement_keeps_capabilities_on_one_pinned_copy() {
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join("ffmpeg-test");
    let replacement = format!(
        "#!/bin/sh\nprintf '#!/bin/sh\\necho replaced\\n' > '{}'\necho 'ffmpeg version pinned-a'\n",
        original.display()
    );
    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then\n{replacement}  rm \"$0\"\n  exit 0\nfi\nif [ \"$2\" = \"-encoders\" ]; then\n  echo ' V..... pinned_encoder test'\n  rm \"$0\"\n  exit 0\nfi\nfor output in \"$@\"; do :; done\nprintf pinned-render > \"$output\"\nrm \"$0\"\n"
    );
    std::fs::write(&original, script).unwrap();
    std::fs::set_permissions(&original, std::fs::Permissions::from_mode(0o755)).unwrap();
    let ffmpeg = SystemFfmpeg::new(&original);
    let clone = ffmpeg.clone();

    assert_eq!(
        ffmpeg.fingerprint().unwrap().version,
        "ffmpeg version pinned-a"
    );
    assert!(std::fs::read_to_string(&original)
        .unwrap()
        .contains("replaced"));
    assert!(clone.encoders().unwrap().contains("pinned_encoder"));
    let output = temp.path().join("rendered");
    let arguments = vec![output.to_string_lossy().into_owned()];
    clone
        .execute(FfmpegInvocation::render(
            &arguments,
            temp.path(),
            Instant::now() + std::time::Duration::from_secs(10),
        ))
        .unwrap();
    assert_eq!(std::fs::read(output).unwrap(), b"pinned-render");
}

#[test]
fn missing_pin_reports_consistent_context_across_clones() {
    let ffmpeg = SystemFfmpeg::new("veac-definitely-missing-tool");
    let clone = ffmpeg.clone();
    let first = ffmpeg.fingerprint().unwrap_err().message;
    let second = clone.encoders().unwrap_err().message;
    assert_eq!(first, second);
    assert!(first.contains("failed to run FFmpeg"));
}

#[test]
fn expired_pin_preserves_resource_kind_without_poisoning_the_cache() {
    let temp = tempfile::tempdir().unwrap();
    let executable = tool(temp.path().join("ffmpeg"), "# deadline retry");
    let ffmpeg = SystemFfmpeg::new(executable);
    let error = ffmpeg.pinned_until(Instant::now()).unwrap_err();
    assert_eq!(error.kind, crate::RuntimeErrorKind::ResourceLimit);
    assert!(error.message.contains("failed to run FFmpeg"));

    assert_eq!(
        ffmpeg.fingerprint().unwrap().version,
        "ffmpeg version identical"
    );
}

#[test]
fn same_version_output_from_different_executables_has_distinct_identity() {
    let temp = tempfile::tempdir().unwrap();
    let first = tool(temp.path().join("first"), "# first bytes");
    let second = tool(temp.path().join("second"), "# second bytes");

    let first = SystemFfmpeg::new(first).fingerprint().unwrap();
    let second = SystemFfmpeg::new(second).fingerprint().unwrap();
    assert_eq!(first.version, second.version);
    assert_ne!(first.configuration, second.configuration);
}

#[test]
fn system_environment_queries_every_capability_table_and_device_list() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("ffmpeg-capabilities");
    let script = r#"#!/bin/sh
case "$*" in
  '-version') printf 'ffmpeg version complete\n' ;;
  '-hide_banner -encoders') printf ' V..... h264 encoder\n' ;;
  '-hide_banner -decoders') printf ' V..... hevc decoder\n' ;;
  '-hide_banner -muxers') printf ' E mp4 muxer\n' ;;
  '-hide_banner -demuxers') printf ' D mov demuxer\n' ;;
  '-hide_banner -filters') printf ' TS scale filter\n' ;;
  '-hide_banner -hwaccels') printf 'videotoolbox\n' ;;
  '-hide_banner -init_hw_device list') printf 'cuda\n' >&2 ;;
esac
"#;
    let ffmpeg = SystemFfmpeg::new(tool_with(&path, script));

    for (kind, expected) in [
        (BackendCapabilityKind::Encoder, "h264"),
        (BackendCapabilityKind::Decoder, "hevc"),
        (BackendCapabilityKind::Muxer, "mp4"),
        (BackendCapabilityKind::Demuxer, "mov"),
        (BackendCapabilityKind::Filter, "scale"),
        (BackendCapabilityKind::HardwareBackend, "videotoolbox"),
        (BackendCapabilityKind::HardwareDevice, "cuda"),
    ] {
        assert!(ffmpeg
            .capability_until(kind, Instant::now() + Duration::from_secs(15))
            .unwrap()
            .contains(expected));
    }
    assert!(ffmpeg.encoders().unwrap().contains("h264"));
    assert!(ffmpeg.decoders().unwrap().contains("hevc"));
    assert!(ffmpeg.muxers().unwrap().contains("mp4"));
    assert!(ffmpeg.demuxers().unwrap().contains("mov"));
    assert!(ffmpeg.filters().unwrap().contains("scale"));
    assert!(ffmpeg.hardware_backends().unwrap().contains("videotoolbox"));
    assert!(ffmpeg.hardware_devices().unwrap().contains("cuda"));
}

#[test]
fn empty_version_and_failed_capability_commands_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let empty = tool_with(&temp.path().join("empty"), "#!/bin/sh\nexit 0\n");
    assert!(SystemFfmpeg::new(empty)
        .fingerprint()
        .unwrap_err()
        .message
        .contains("empty version"));
    let failed = tool_with(
        &temp.path().join("failed"),
        "#!/bin/sh\nprintf denied >&2\nexit 4\n",
    );
    let error = SystemFfmpeg::new(failed).filters().unwrap_err();
    assert!(error.message.contains("filter probe failed: denied"));
}

fn tool(path: PathBuf, comment: &str) -> PathBuf {
    tool_with(
        &path,
        &format!("#!/bin/sh\n{comment}\nprintf 'ffmpeg version identical\\n'\n"),
    )
}

fn tool_with(path: &std::path::Path, contents: &str) -> PathBuf {
    std::fs::write(path, contents).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
    path.to_path_buf()
}
