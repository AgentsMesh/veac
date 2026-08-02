#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use veac_artifact::*;
use veac_ir::{Rational, RationalTime, StreamSelection};
use veac_runtime::workflow::{MediaWorkflow, WorkflowErrorKind};

use super::support::*;

#[test]
fn source_range_beyond_the_selected_stream_fails_before_ffmpeg() {
    let fixture = Fixture::new();
    let spec = proxy(SourceClockSpec::Identity {
        duration: time(200),
    });
    let request = request_with_tool(&fixture.input, spec, fixture.tool.clone());
    let store_root = fixture.temp.path().join("range-store");
    let error = fixture
        .workflow()
        .derive(&ArtifactStore::new(&store_root), &fixture.input, &request)
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::InvalidContract);
    assert!(!fixture.marker.exists());
    assert!(!store_root.exists());
}

#[test]
fn matching_descriptor_never_trusts_malicious_cached_payload_bytes() {
    let fixture = Fixture::new();
    let request = request_with_tool(&fixture.input, proxy(identity()), fixture.tool.clone());
    let descriptor = request.descriptor().unwrap();
    let store = ArtifactStore::new(fixture.temp.path().join("cache-store"));
    store.put(&descriptor, b"not a video artifact").unwrap();

    let error = fixture
        .workflow()
        .derive(&store, &fixture.input, &request)
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ToolFailure);
    assert!(!fixture.marker.exists());
}

#[test]
fn active_low_work_limit_rejects_before_ffmpeg_and_store() {
    let fixture = Fixture::new();
    let request = request_with_tool(&fixture.input, proxy(identity()), fixture.tool.clone());
    let limits = MediaArtifactLimits {
        max_video_frames: 1,
        ..MediaArtifactLimits::default()
    };
    let store_root = fixture.temp.path().join("limit-store");
    let error = fixture
        .workflow()
        .with_limits(limits)
        .derive(&ArtifactStore::new(&store_root), &fixture.input, &request)
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ResourceLimit);
    assert!(!fixture.marker.exists());
    assert!(!store_root.exists());
}

struct Fixture {
    temp: tempfile::TempDir,
    input: PathBuf,
    tool: PathBuf,
    marker: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let input = media_fixture(temp.path());
        let marker = temp.path().join("ffmpeg-rendered");
        let tool = wrapper(temp.path(), &marker);
        Self {
            temp,
            input,
            tool,
            marker,
        }
    }

    fn workflow(&self) -> MediaWorkflow {
        MediaWorkflow::new(&self.tool)
    }
}

fn proxy(source_clock: SourceClockSpec) -> MediaArtifactSpec {
    MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
        source_stream: StreamSelection {
            global_index: 1,
            type_index: 1,
        },
        source_clock,
        width: 120,
        height: 68,
        frame_rate: Rational::new(12, 1).unwrap(),
        crf: 26,
    })
}

fn identity() -> SourceClockSpec {
    SourceClockSpec::Identity {
        duration: time(100),
    }
}

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 100).unwrap()
}

fn wrapper(root: &Path, marker: &Path) -> PathBuf {
    let path = root.join("ffmpeg-wrapper.sh");
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then exec ffmpeg \"$@\"; fi\ntouch '{}'\nexec ffmpeg \"$@\"\n",
            marker.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    path
}
