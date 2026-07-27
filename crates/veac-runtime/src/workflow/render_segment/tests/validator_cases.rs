use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Instant;

use super::support::*;
use super::*;

#[test]
fn validator_runs_the_bounded_payload_probe_and_snapshot_pipeline() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("segment.bin");
    let bytes = b"render segment";
    std::fs::write(&path, bytes).unwrap();
    let segment = contract(false);
    let mut expected = record(&segment);
    expected.size_bytes = bytes.len() as u64;
    let probe = probe_script(temp.path(), &segment);
    let validator =
        FullRenderSegmentValidator::with_limits(SystemFfprobe::new(probe), 30, bytes.len() as u64)
            .unwrap();

    validator.validate(&path, &segment, &expected).unwrap();
}

#[test]
fn validator_constructor_deadline_and_probe_failures_are_typed() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("segment.bin");
    let bytes = b"render segment";
    std::fs::write(&path, bytes).unwrap();
    let segment = contract(false);
    let mut expected = record(&segment);
    expected.size_bytes = bytes.len() as u64;
    let validator =
        FullRenderSegmentValidator::new(SystemFfprobe::new(temp.path().join("missing-ffprobe")));

    let expired = validator
        .validate_until(&path, &segment, &expected, Instant::now())
        .unwrap_err();
    assert_eq!(expired.kind, WorkflowErrorKind::ResourceLimit);
    let tool = validator.validate(&path, &segment, &expected).unwrap_err();
    assert_eq!(tool.kind, WorkflowErrorKind::ToolFailure);
}

fn probe_script(root: &Path, segment: &FullRenderSegmentContract) -> std::path::PathBuf {
    let profile = segment.media_profile();
    let duration = segment.range().duration;
    let seconds = duration.value as f64 / f64::from(duration.timescale);
    let rate = profile.frame_rate();
    let output = serde_json::json!({
        "format": {
            "format_name": "mov,mp4,m4a,3gp,3g2,mj2",
            "duration": format!("{seconds:.12}"),
            "tags": { "major_brand": "isom" }
        },
        "streams": [{
            "index": 0, "codec_type": "video", "codec_name": "h264",
            "time_base": format!("1/{}", duration.timescale),
            "avg_frame_rate": format!("{}/{}", rate.numerator, rate.denominator),
            "r_frame_rate": format!("{}/{}", rate.numerator, rate.denominator),
            "start_time": "0", "duration": format!("{seconds:.12}"),
            "width": profile.width(), "height": profile.height(), "pix_fmt": "yuv420p",
            "profile": "High", "level": 40, "sample_aspect_ratio": "1:1",
            "disposition": { "default": 1, "attached_pic": 0, "timed_thumbnails": 0 }
        }]
    });
    let path = root.join("ffprobe.sh");
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then echo 'ffprobe version test-1'; exit 0; fi\nprintf '%s' '{}'\n",
            output.to_string().replace('\'', "'\\''")
        ),
    )
    .unwrap();
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).unwrap();
    path
}
