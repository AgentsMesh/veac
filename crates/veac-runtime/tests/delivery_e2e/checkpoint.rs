use std::collections::BTreeSet;

use tempfile::tempdir;
use veac_artifact::{ArtifactStore, ContentDigest};
use veac_runtime::executor::{
    BundleExecutor, FfmpegEnvironment, FfmpegFingerprint, FfmpegInvocation,
};
use veac_runtime::RuntimeError;

use super::support::*;

#[test]
fn real_two_pass_render_resumes_and_revalidates_logs_and_master() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks.push(track(
        "trk_source",
        TrackKind::Video,
        0,
        vec![solid_clip("itm_source", color(20, 100, 220), 0, 1_000)],
    ));
    let deliverable_id = canonical.project.render_configs[0].deliverables[0]
        .id
        .clone();
    let video = canonical.project.render_configs[0]
        .video_deliverable_mut(&deliverable_id)
        .unwrap();
    video.pass_mode = PassMode::TwoPass;
    video.hardware = HardwareSelection::Software;
    video.video.rate_control = VideoRateControl::Bitrate {
        target_bps: 250_000,
        max_bps: Some(350_000),
        buffer_size_bits: Some(500_000),
    };
    let delivery = prepare_delivery(canonical, &BTreeMap::new(), temp.path());
    let store = ArtifactStore::new(temp.path().join("store"));
    let first = delivery.execute(&store);
    assert_eq!(first.tasks.len(), 2);
    let log = first.tasks[0]
        .outputs
        .iter()
        .find(|path| path.to_string_lossy().ends_with("-0.log"))
        .unwrap()
        .clone();
    assert!(log.is_file());
    assert!(delivery.path("dlv_main").is_file());
    let hit = delivery.execute(&store);
    assert!(hit.tasks.iter().all(|task| task.cache_hit));

    std::fs::write(&log, b"corrupt passlog").unwrap();
    let repaired = delivery.execute(&store);
    assert!(!repaired.tasks[0].cache_hit);
    let same_dependency = repaired.tasks[0].checkpoint.content == first.tasks[0].checkpoint.content;
    assert_eq!(repaired.tasks[1].cache_hit, same_dependency);
    assert_ne!(std::fs::read(&log).unwrap(), b"corrupt passlog");

    std::fs::write(delivery.path("dlv_main"), b"corrupt master").unwrap();
    let master_repaired = delivery.execute(&store);
    assert!(master_repaired.tasks[0].cache_hit);
    assert!(!master_repaired.tasks[1].cache_hit);
    assert!(std::fs::metadata(delivery.path("dlv_main")).unwrap().len() > 1_000);
}

#[test]
fn sealed_bundle_encoder_preflight_rejects_unavailable_encoder() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks.push(track(
        "trk_source",
        TrackKind::Video,
        0,
        vec![solid_clip("itm_source", color(20, 100, 220), 0, 1_000)],
    ));
    let delivery = prepare_delivery(canonical, &BTreeMap::new(), temp.path());
    let error = BundleExecutor::new(NoEncoders)
        .execute(
            &delivery.bundle,
            &ArtifactStore::new(temp.path().join("store")),
        )
        .unwrap_err();
    assert!(error.message.contains("libx264"));
    assert!(!delivery.path("dlv_main").exists());
}

struct NoEncoders;

impl FfmpegEnvironment for NoEncoders {
    fn fingerprint(&self) -> Result<FfmpegFingerprint, RuntimeError> {
        Ok(FfmpegFingerprint {
            version: "ffmpeg test".to_owned(),
            configuration: ContentDigest::sha256(b"test"),
        })
    }

    fn encoders(&self) -> Result<BTreeSet<String>, RuntimeError> {
        Ok(BTreeSet::new())
    }

    fn muxers(&self) -> Result<BTreeSet<String>, RuntimeError> {
        Ok(BTreeSet::new())
    }

    fn execute(&self, _: FfmpegInvocation<'_>) -> Result<(), RuntimeError> {
        panic!("encoder preflight must stop before execution")
    }
}
