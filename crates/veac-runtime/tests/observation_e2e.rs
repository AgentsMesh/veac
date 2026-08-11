use std::process::Command;

use tempfile::tempdir;
use veac_ir::RationalTime;
use veac_runtime::asset::sha256_identity;
use veac_runtime::observation::{
    DecodeRequest, FramePixelFormat, FrameRequest, MediaObserver, ObservationSource,
};

#[test]
fn observer_decodes_an_identity_bound_frame_and_complete_stream() {
    let temp = tempdir().unwrap();
    let media = temp.path().join("fixture.mkv");
    let output = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=4x2:r=2:d=1",
            "-c:v",
            "ffv1",
            "-threads",
            "1",
            "-y",
        ])
        .arg(&media)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let source = ObservationSource {
        identity: sha256_identity(&media).unwrap(),
        path: media,
        video_stream: None,
    };
    let observer = MediaObserver::default();
    let frame = observer
        .frame(&FrameRequest {
            source: source.clone(),
            time: RationalTime::zero(2).unwrap(),
            pixel_format: FramePixelFormat::Rgba8,
        })
        .unwrap();
    assert_eq!((frame.width, frame.height), (4, 2));
    assert_eq!(frame.actual_pts, RationalTime::zero(1_000).unwrap());
    assert_eq!(frame.bytes.len(), 32);
    assert!(frame
        .bytes
        .chunks_exact(4)
        .all(|pixel| pixel[0] > 240 && pixel[3] == 255));
    let decoded = observer
        .decode(&DecodeRequest {
            source: source.clone(),
        })
        .unwrap();
    assert!(decoded.complete);
    assert_eq!(decoded.decoded_frames, 2);
    assert!(decoded.last_pts.is_some());
    assert!(decoded.errors.is_empty());

    let error = observer
        .frame(&FrameRequest {
            source,
            time: RationalTime::zero(2).unwrap(),
            pixel_format: FramePixelFormat::Alpha16,
        })
        .unwrap_err();
    assert!(error.message.contains("source alpha plane"));
}

#[test]
fn observer_reports_the_selected_pts_for_variable_frame_cadence() {
    let temp = tempdir().unwrap();
    let media = temp.path().join("vfr.mkv");
    let output = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "nullsrc=s=4x2:r=10:d=0.3,geq=r='N*80':g=0:b=0",
            "-vf",
            "setpts='if(eq(N,0),0,if(eq(N,1),2,7))/(10*TB)'",
            "-fps_mode",
            "vfr",
            "-c:v",
            "ffv1",
            "-threads",
            "1",
            "-y",
        ])
        .arg(&media)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let source = ObservationSource {
        identity: sha256_identity(&media).unwrap(),
        path: media,
        video_stream: None,
    };
    let frame = MediaObserver::default()
        .frame(&FrameRequest {
            source,
            time: RationalTime::new(15, 100).unwrap(),
            pixel_format: FramePixelFormat::Rgba8,
        })
        .unwrap();
    assert_eq!(
        frame
            .actual_pts
            .partial_cmp(&RationalTime::new(1, 5).unwrap()),
        Some(std::cmp::Ordering::Equal)
    );
    assert!(frame
        .bytes
        .chunks_exact(4)
        .all(|pixel| pixel == [80, 0, 0, 255]));
}
