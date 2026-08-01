use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use veac_artifact::ContentDigest;
use veac_codegen::emitter::{
    BackendAction, BackendCommand, BackendOutput, BackendPackagePaths, BackendPhase,
    BackendProduct, BackendTask,
};

use super::super::{package, perform, StagedOutput};
use super::deadline;
use crate::executor::{FfmpegEnvironment, FfmpegFingerprint, FfmpegInvocation};

#[test]
fn hls_inventory_requires_a_closed_local_vod_graph() {
    let root = tempfile::tempdir().unwrap();
    write_hls(root.path());
    let inventory =
        package::inspect_hls(root.path(), Path::new("master.m3u8"), deadline()).unwrap();
    assert_eq!(inventory.entrypoint, "master.m3u8");
    assert_eq!(inventory.members.len(), 3);

    std::fs::write(root.path().join("orphan.ts"), b"orphan").unwrap();
    assert!(
        package::inspect_hls(root.path(), Path::new("master.m3u8"), deadline())
            .unwrap_err()
            .message
            .contains("unreachable")
    );
    std::fs::remove_file(root.path().join("orphan.ts")).unwrap();

    std::fs::write(
        root.path().join("master.m3u8"),
        "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1000\nhttps://example.test/media.m3u8\n",
    )
    .unwrap();
    assert!(
        package::inspect_hls(root.path(), Path::new("master.m3u8"), deadline())
            .unwrap_err()
            .message
            .contains("local relative")
    );
}

#[test]
fn package_staging_uses_private_cwd_and_atomically_replaces_a_tree() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("stream");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("old.ts"), b"old").unwrap();
    let environment = HlsFfmpeg::default();
    let staged = perform(&environment, &task(&target), deadline()).unwrap();
    let working = environment.working.borrow().clone().unwrap();
    assert!(working.ends_with("package"));
    assert_eq!(staged.outputs().len(), 1);
    assert!(matches!(staged.outputs()[0], StagedOutput::Package(_)));

    let parent = std::fs::canonicalize(root.path()).unwrap();
    let locks = crate::executor::locking::acquire_until(&[parent], deadline()).unwrap();
    staged.commit(&locks, deadline()).unwrap();
    assert!(!target.join("old.ts").exists());
    assert!(target.join("master.m3u8").is_file());
    assert!(target.join("segment-rnd_main-000000.ts").is_file());
}

fn task(target: &Path) -> BackendTask {
    let paths = BackendPackagePaths {
        playlist_pattern: "rendition-%v.m3u8".into(),
        segment_pattern: "segment-%v-%06d.ts".into(),
    };
    BackendTask {
        deliverable_id: veac_ir::DeliverableId::new("dlv_hls_test").unwrap(),
        phase: BackendPhase::Single,
        product: BackendProduct::HlsVod,
        output: BackendOutput::Package {
            root: target.to_path_buf(),
            entrypoint: "master.m3u8".into(),
            paths: paths.clone(),
        },
        action: BackendAction::Ffmpeg(BackendCommand {
            preparations: vec![],
            inputs: vec![],
            filter_graph: None,
            filter_contract: None,
            maps: vec![],
            output_args: vec![
                "-f".into(),
                "hls".into(),
                "-hls_playlist_type".into(),
                "vod".into(),
                "-hls_list_size".into(),
                "0".into(),
                "-master_pl_name".into(),
                "master.m3u8".into(),
                "-hls_segment_filename".into(),
                "segment-%v-%06d.ts".into(),
            ],
            output_path: paths.playlist_pattern,
        }),
    }
}

#[derive(Default)]
struct HlsFfmpeg {
    working: RefCell<Option<PathBuf>>,
}

impl FfmpegEnvironment for HlsFfmpeg {
    fn fingerprint(&self) -> Result<FfmpegFingerprint, crate::RuntimeError> {
        Ok(FfmpegFingerprint {
            version: "fixture".into(),
            configuration: ContentDigest::sha256(b"fixture"),
        })
    }

    fn encoders(&self) -> Result<BTreeSet<String>, crate::RuntimeError> {
        Ok(BTreeSet::new())
    }

    fn muxers(&self) -> Result<BTreeSet<String>, crate::RuntimeError> {
        Ok(BTreeSet::new())
    }

    fn execute(&self, invocation: FfmpegInvocation<'_>) -> Result<(), crate::RuntimeError> {
        let working = invocation
            .working_directory()
            .ok_or_else(|| crate::RuntimeError::new("missing FFmpeg cwd"))?;
        self.working.replace(Some(working.to_path_buf()));
        write_hls(working);
        Ok(())
    }
}

fn write_hls(root: &Path) {
    std::fs::write(
        root.join("master.m3u8"),
        "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1000\nrendition-rnd_main.m3u8\n",
    )
    .unwrap();
    std::fs::write(
        root.join("rendition-rnd_main.m3u8"),
        concat!(
            "#EXTM3U\n#EXT-X-TARGETDURATION:1\n#EXT-X-PLAYLIST-TYPE:VOD\n",
            "#EXTINF:1.0,\nsegment-rnd_main-000000.ts\n#EXT-X-ENDLIST\n"
        ),
    )
    .unwrap();
    std::fs::write(root.join("segment-rnd_main-000000.ts"), b"segment").unwrap();
}
