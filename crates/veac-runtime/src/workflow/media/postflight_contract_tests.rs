#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};

use veac_artifact::*;

use super::*;

#[test]
fn fresh_artifact_cannot_exceed_its_declared_duration() {
    let fixture = Fixture::new();
    let store_root = fixture.temp.path().join("fresh-store");
    let error = fixture
        .workflow()
        .derive(
            &ArtifactStore::new(&store_root),
            &fixture.input,
            &fixture.request,
        )
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ToolFailure);
    assert!(error.to_string().contains("duration"));
    assert!(!store_root.exists());
}

#[test]
fn cached_artifact_receives_the_same_duration_postflight() {
    let fixture = Fixture::new();
    let store = ArtifactStore::new(fixture.temp.path().join("cache-store"));
    let descriptor = fixture.request.descriptor().unwrap();
    store.put(&descriptor, b"cached-untrusted-bytes").unwrap();

    let error = fixture
        .workflow()
        .derive(&store, &fixture.input, &fixture.request)
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ToolFailure);
    assert!(error.to_string().contains("duration"));
    assert!(!fixture.marker.exists());
}

struct Fixture {
    temp: tempfile::TempDir,
    input: PathBuf,
    ffmpeg: PathBuf,
    ffprobe: PathBuf,
    marker: PathBuf,
    request: MediaArtifactRequest,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let input = temp.path().join("input");
        fs::write(&input, b"source").unwrap();
        let marker = temp.path().join("ffmpeg-action");
        let ffmpeg = fake_ffmpeg(temp.path(), &marker);
        let ffprobe = fake_ffprobe(temp.path());
        let request = request(&ffmpeg);
        Self {
            temp,
            input,
            ffmpeg,
            ffprobe,
            marker,
            request,
        }
    }

    fn workflow(&self) -> MediaWorkflow {
        MediaWorkflow::with_tools(&self.ffmpeg, &self.ffprobe)
    }
}

fn request(ffmpeg: &Path) -> MediaArtifactRequest {
    MediaArtifactRequest {
        source_identity: ContentDigest::sha256(b"source"),
        producer: super::security_tests::ffmpeg_producer(ffmpeg),
        spec: MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream: veac_ir::StreamSelection {
                global_index: 0,
                type_index: 0,
            },
            source_clock: SourceClockSpec::Identity {
                duration: veac_ir::RationalTime::new(1, 1).unwrap(),
            },
            sample_rate: 48_000,
            channels: 2,
        }),
    }
}

fn fake_ffmpeg(root: &Path, marker: &Path) -> PathBuf {
    super::test_support::executable(
        root.join("duration-ffmpeg.sh"),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then echo 'ffmpeg version duration'; exit 0; fi\ntouch '{}'\nfor output in \"$@\"; do :; done\nprintf artifact > \"$output\"\n",
            marker.display()
        ),
    )
}

fn fake_ffprobe(root: &Path) -> PathBuf {
    super::test_support::executable(
        root.join("duration-ffprobe.sh"),
        "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then echo 'ffprobe version duration'; exit 0; fi\nprintf '%s' '{\"format\":{\"format_name\":\"wav\",\"duration\":\"2\"},\"streams\":[{\"index\":0,\"codec_type\":\"audio\",\"codec_name\":\"pcm_s16le\",\"time_base\":\"1/48000\",\"start_time\":\"0\",\"duration\":\"2\",\"sample_rate\":\"48000\",\"channels\":2,\"channel_layout\":\"stereo\"}]}'\n",
    )
}
