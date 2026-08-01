use std::path::{Component, Path, PathBuf};
use std::process::Command;

use m3u8_rs::{MediaPlaylistType, Playlist};

pub(super) struct HlsGraph {
    pub media_playlists: Vec<PathBuf>,
    pub segments: Vec<PathBuf>,
}

pub(super) fn inspect(root: &Path) -> HlsGraph {
    let master_path = root.join("master.m3u8");
    let Playlist::MasterPlaylist(master) = parse(&master_path) else {
        panic!("entrypoint is not an HLS master playlist")
    };
    assert!(!master.variants.is_empty());
    let mut media_playlists = Vec::new();
    let mut segments = Vec::new();
    for variant in master.variants {
        let media_path = local(root, &variant.uri);
        let Playlist::MediaPlaylist(media) = parse(&media_path) else {
            panic!("master URI is not a media playlist: {}", variant.uri)
        };
        assert_eq!(media.playlist_type, Some(MediaPlaylistType::Vod));
        assert!(media.end_list && !media.segments.is_empty());
        for segment in media.segments {
            let segment_path = local(root, &segment.uri);
            assert!(segment_path.is_file(), "missing {}", segment_path.display());
            segments.push(segment_path);
        }
        media_playlists.push(media_path);
    }
    media_playlists.sort();
    media_playlists.dedup();
    segments.sort();
    segments.dedup();
    HlsGraph {
        media_playlists,
        segments,
    }
}

pub(super) fn assert_segment_decodes(path: &Path, with_audio: bool) {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "stream=codec_name,codec_type",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .expect("probe HLS segment");
    assert!(
        output.status.success(),
        "segment {} is not decodable: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let streams = value["streams"].as_array().unwrap();
    assert!(streams.iter().any(|stream| stream["codec_name"] == "h264"));
    assert_eq!(
        streams.iter().any(|stream| stream["codec_name"] == "aac"),
        with_audio
    );
}

fn parse(path: &Path) -> Playlist {
    let bytes =
        std::fs::read(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    m3u8_rs::parse_playlist_res(&bytes).unwrap_or_else(|_| panic!("parse {}", path.display()))
}

fn local(root: &Path, uri: &str) -> PathBuf {
    let path = Path::new(uri);
    assert!(
        !uri.is_empty()
            && path
                .components()
                .all(|component| matches!(component, Component::Normal(_))),
        "unsafe HLS URI {uri}"
    );
    let joined = root.join(path);
    assert!(joined.is_file(), "missing local HLS URI {uri}");
    joined
}
