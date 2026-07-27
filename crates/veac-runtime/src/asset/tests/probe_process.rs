#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use tempfile::tempdir;

use super::*;

#[test]
fn successful_probe_rejects_media_changed_during_observation() {
    let temp = tempdir().unwrap();
    let media = media(temp.path());
    let probe = script(
        temp.path(),
        "mutating.sh",
        &format!(
            "printf changed > '{}'\nprintf '%s' '{{\"streams\":[],\"format\":{{\"format_name\":\"data\"}}}}'",
            media.display()
        ),
    );

    let error = run(&media, &probe).unwrap_err();
    assert!(matches!(error, ProbeError::IdentityChanged { path } if path == media));
    assert_eq!(std::fs::read(media).unwrap(), b"changed");
}

#[test]
fn successful_probe_rejects_swapped_symlink_fifo_and_regular_paths() {
    let temp = tempdir().unwrap();
    for (name, replacement) in [
        (
            "symlink",
            "mv '{0}' '{0}.original'; ln -s '{0}.original' '{0}'",
        ),
        ("fifo", "rm '{0}'; mkfifo '{0}'"),
        ("regular", "mv '{0}' '{0}.original'; printf changed > '{0}'"),
    ] {
        let root = temp.path().join(name);
        std::fs::create_dir(&root).unwrap();
        let media = media(&root);
        let action = replacement.replace("{0}", &media.to_string_lossy());
        let body = format!(
            "{action}\nprintf '%s' '{{\"streams\":[],\"format\":{{\"format_name\":\"data\"}}}}'"
        );
        let probe = script(&root, "swap.sh", &body);

        let error = run(&media, &probe).unwrap_err();
        assert!(matches!(error, ProbeError::IdentityChanged { path } if path == media));
    }
}

#[test]
fn failed_probe_keeps_process_failure_priority_over_identity_change() {
    let temp = tempdir().unwrap();
    let media = media(temp.path());
    let probe = script(
        temp.path(),
        "failed.sh",
        &format!("printf changed > '{}'\nexit 9", media.display()),
    );

    assert!(matches!(
        run(&media, &probe),
        Err(ProbeError::ProcessFailed {
            status: Some(9),
            ..
        })
    ));
}

#[test]
fn stable_identity_reaches_probe_json_validation() {
    let temp = tempdir().unwrap();
    let media = media(temp.path());
    let probe = script(temp.path(), "invalid-json.sh", "printf not-json");

    assert!(matches!(
        run(&media, &probe),
        Err(ProbeError::InvalidJson { .. })
    ));
    assert_eq!(std::fs::read(media).unwrap(), b"original");
}

#[test]
fn transient_source_swap_and_self_deleting_probe_consume_snapshot_a() {
    let temp = tempdir().unwrap();
    let media = media(temp.path());
    let body = format!(
        r#"for argument in "$@"; do snapshot="$argument"; done
printf version-b > '{}'
bytes=$(cat "$snapshot")
printf original > '{}'
if [ "$bytes" = original ]; then
  printf '%s' '{{"streams":[],"format":{{"format_name":"data"}}}}'
else
  printf not-json
fi"#,
        media.display(),
        media.display()
    );
    let probe = self_deleting_script(temp.path(), "swap.sh", &body);

    let snapshot = run(&media, &probe).unwrap();
    assert_eq!(snapshot.observed_identity, sha256_identity(&media).unwrap());
    assert_eq!(std::fs::read(media).unwrap(), b"original");
}

#[test]
fn same_version_different_probe_bytes_have_distinct_engine_identity() {
    let temp = tempdir().unwrap();
    let media = media(temp.path());
    let json = "printf '%s' '{\"streams\":[],\"format\":{\"format_name\":\"data\"}}'";
    let first = script(temp.path(), "first.sh", &format!("# first bytes\n{json}"));
    let second = script(temp.path(), "second.sh", &format!("# second bytes\n{json}"));

    let first = run(&media, &first).unwrap();
    let second = run(&media, &second).unwrap();
    assert!(first.engine.starts_with("ffprobe version test-1 sha256:"));
    assert!(second.engine.starts_with("ffprobe version test-1 sha256:"));
    assert_ne!(first.engine, second.engine);
}

fn run(media: &Path, probe: &Path) -> Result<MediaProbeSnapshot, ProbeError> {
    SystemFfprobe::new(probe).probe_with_intent(media, auto_stream_intent())
}

fn media(root: &Path) -> PathBuf {
    let path = root.join("media.bin");
    std::fs::write(&path, b"original").unwrap();
    path
}

fn script(root: &Path, name: &str, body: &str) -> PathBuf {
    let path = root.join(name);
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then\n  echo \"ffprobe version test-1\"\n  exit 0\nfi\n{body}\n"
        ),
    )
    .unwrap();
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).unwrap();
    path
}

fn self_deleting_script(root: &Path, name: &str, body: &str) -> PathBuf {
    let path = root.join(name);
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then\n  echo \"ffprobe version test-1\"\n  rm \"$0\"\n  exit 0\nfi\n{body}\n"
        ),
    )
    .unwrap();
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).unwrap();
    path
}
