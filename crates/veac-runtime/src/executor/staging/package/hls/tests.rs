use std::path::Path;

use super::*;

const MASTER: &str = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1000\nmedia.m3u8\n";
const MEDIA: &str = concat!(
    "#EXTM3U\n#EXT-X-TARGETDURATION:1\n#EXT-X-PLAYLIST-TYPE:VOD\n",
    "#EXTINF:1.0,\nsegment.ts\n#EXT-X-ENDLIST\n"
);

#[test]
fn rejects_non_master_and_malformed_entrypoints() {
    assert_error(
        "media.m3u8",
        &[("media.m3u8", MEDIA), ("segment.ts", "segment")],
        "master playlist",
    );
    assert_error("master.m3u8", &[("master.m3u8", "not m3u8")], "invalid HLS");
}

#[test]
fn rejects_empty_unsupported_and_duplicate_master_graphs() {
    let alternative = concat!(
        "#EXTM3U\n",
        "#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"a\",NAME=\"a\",URI=\"audio.m3u8\"\n"
    );
    assert_error(
        "master.m3u8",
        &[("master.m3u8", alternative)],
        "unsupported or empty",
    );
    let duplicate = concat!(
        "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1000\nmedia.m3u8\n",
        "#EXT-X-STREAM-INF:BANDWIDTH=2000\nmedia.m3u8\n"
    );
    assert_error(
        "master.m3u8",
        &[
            ("master.m3u8", duplicate),
            ("media.m3u8", MEDIA),
            ("segment.ts", "segment"),
        ],
        "duplicate media",
    );
}

#[test]
fn rejects_nested_master_missing_segment_and_escaping_uri() {
    assert_error(
        "master.m3u8",
        &[("master.m3u8", MASTER), ("media.m3u8", MASTER)],
        "non-media",
    );
    assert_error(
        "master.m3u8",
        &[("master.m3u8", MASTER), ("media.m3u8", MEDIA)],
        "missing package member",
    );
    let escaping = MEDIA.replace("segment.ts", "../segment.ts");
    assert_error(
        "master.m3u8",
        &[("master.m3u8", MASTER), ("media.m3u8", &escaping)],
        "must not escape",
    );
}

#[test]
fn requires_nonempty_vod_media_with_endlist() {
    let cases = [
        MEDIA.replace("#EXT-X-PLAYLIST-TYPE:VOD\n", ""),
        MEDIA.replace("#EXT-X-ENDLIST\n", ""),
        MEDIA.replace("#EXTINF:1.0,\nsegment.ts\n", ""),
    ];
    for media in cases {
        assert_error(
            "master.m3u8",
            &[("master.m3u8", MASTER), ("media.m3u8", &media)],
            "non-empty VOD with ENDLIST",
        );
    }
}

#[test]
fn rejects_external_segment_resources() {
    for tag in [
        "#EXT-X-BYTERANGE:4@0\n",
        "#EXT-X-KEY:METHOD=AES-128,URI=\"key.bin\"\n",
        "#EXT-X-MAP:URI=\"init.mp4\"\n",
    ] {
        let media = MEDIA.replace("#EXTINF:1.0,\n", &format!("{tag}#EXTINF:1.0,\n"));
        assert_error(
            "master.m3u8",
            &[
                ("master.m3u8", MASTER),
                ("media.m3u8", &media),
                ("segment.ts", "segment"),
            ],
            "external segment resource",
        );
    }
}

#[test]
fn accepts_a_closed_nested_graph() {
    let master = MASTER.replace("media.m3u8", "streams/media.m3u8");
    let media = MEDIA.replace("segment.ts", "chunks/segment.ts");
    let result = validate_files(
        "master.m3u8",
        &[
            ("master.m3u8", &master),
            ("streams/media.m3u8", &media),
            ("streams/chunks/segment.ts", "segment"),
        ],
    );
    assert!(result.is_ok(), "{result:?}");
}

fn assert_error(entrypoint: &str, files: &[(&str, &str)], expected: &str) {
    let error = validate_files(entrypoint, files).unwrap_err();
    assert!(error.message.contains(expected), "{error}");
}

fn validate_files(entrypoint: &str, files: &[(&str, &str)]) -> Result<(), RuntimeError> {
    let temp = tempfile::tempdir().unwrap();
    for (path, content) in files {
        let path = temp.path().join(path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }
    let root = Directory::open(temp.path())?;
    let inventory = super::super::scanner::inventory(&root, Path::new(entrypoint), limit())?;
    validate(&root, &inventory, limit())
}

fn limit() -> Instant {
    Instant::now() + std::time::Duration::from_secs(10)
}
