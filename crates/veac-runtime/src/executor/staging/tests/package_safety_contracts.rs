#![cfg(unix)]

use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::symlink;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::time::Instant;

use super::super::package;
use super::deadline;

#[test]
fn package_inventory_rejects_links_special_nodes_and_non_utf8_names() {
    for attack in [
        Attack::Symlink,
        Attack::Hardlink,
        Attack::Socket,
        Attack::NonUtf8,
    ] {
        let root = tempfile::tempdir().unwrap();
        write_hls(root.path());
        if !install(root.path(), attack) {
            continue;
        }
        let error =
            package::inspect_hls(root.path(), Path::new("master.m3u8"), deadline()).unwrap_err();
        assert!(
            error.message.contains("package")
                || error.message.contains("descriptor-relative")
                || error.message.contains("UTF-8"),
            "{attack:?}: {error}"
        );
    }
}

#[test]
fn package_inventory_rejects_empty_directories_and_expired_work() {
    let root = tempfile::tempdir().unwrap();
    write_hls(root.path());
    std::fs::create_dir(root.path().join("empty")).unwrap();
    assert!(
        package::inspect_hls(root.path(), Path::new("master.m3u8"), deadline())
            .unwrap_err()
            .message
            .contains("empty directory")
    );
    assert_eq!(
        package::inspect_hls(root.path(), Path::new("master.m3u8"), Instant::now())
            .unwrap_err()
            .kind,
        crate::RuntimeErrorKind::ResourceLimit
    );
}

#[derive(Debug, Clone, Copy)]
enum Attack {
    Symlink,
    Hardlink,
    Socket,
    NonUtf8,
}

fn install(root: &Path, attack: Attack) -> bool {
    match attack {
        Attack::Symlink => symlink("master.m3u8", root.join("attack")).is_ok(),
        Attack::Hardlink => {
            std::fs::hard_link(root.join("master.m3u8"), root.join("attack")).is_ok()
        }
        Attack::Socket => UnixListener::bind(root.join("attack")).is_ok(),
        Attack::NonUtf8 => {
            std::fs::write(root.join(OsString::from_vec(vec![0xff])), b"attack").is_ok()
        }
    }
}

fn write_hls(root: &Path) {
    std::fs::write(
        root.join("master.m3u8"),
        "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1000\nmedia.m3u8\n",
    )
    .unwrap();
    std::fs::write(
        root.join("media.m3u8"),
        concat!(
            "#EXTM3U\n#EXT-X-TARGETDURATION:1\n#EXT-X-PLAYLIST-TYPE:VOD\n",
            "#EXTINF:1.0,\nsegment.ts\n#EXT-X-ENDLIST\n"
        ),
    )
    .unwrap();
    std::fs::write(root.join("segment.ts"), b"segment").unwrap();
}
