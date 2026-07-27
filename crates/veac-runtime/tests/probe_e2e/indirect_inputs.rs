use std::path::Path;

use veac_runtime::asset::{probe, ProbeError};

#[test]
fn real_probe_rejects_hls_child_resources() {
    let temp = tempfile::tempdir().unwrap();
    let manifest = temp.path().join("playlist.m3u8");
    std::fs::write(
        &manifest,
        "#EXTM3U\n#EXT-X-TARGETDURATION:1\n#EXTINF:1,\nchild.ts\n#EXT-X-ENDLIST\n",
    )
    .unwrap();
    std::fs::write(temp.path().join("child.ts"), b"not opened").unwrap();
    assert_rejected(&manifest);
}

#[test]
fn real_probe_rejects_ffconcat_child_resources() {
    let temp = tempfile::tempdir().unwrap();
    let manifest = temp.path().join("files.ffconcat");
    std::fs::write(&manifest, "ffconcat version 1.0\nfile child.wav\n").unwrap();
    std::fs::write(temp.path().join("child.wav"), b"not opened").unwrap();
    assert_rejected(&manifest);
}

fn assert_rejected(path: &Path) {
    let error = probe(path).unwrap_err();
    let ProbeError::ProcessFailed { stderr, .. } = error else {
        panic!("unexpected probe error: {error:?}");
    };
    assert!(stderr.contains("Format not on whitelist"), "{stderr}");
}
