use std::path::Path;

use veac_artifact::ArtifactStore;
use veac_codegen::emitter::{
    BackendAction, BackendCommand, BackendOutput, BackendPackagePaths, BackendPhase,
    BackendProduct, BackendTask,
};

use super::super::{resume, store};
use super::support::{deadline, identity_for};
use crate::executor::staging::{self, StagedOutput, StagedPackage};

#[test]
fn package_checkpoint_reuses_only_the_exact_published_tree() {
    let temp = tempfile::tempdir().unwrap();
    let staged = temp.path().join("staged");
    let published = temp.path().join("stream");
    std::fs::create_dir(&staged).unwrap();
    std::fs::create_dir(&published).unwrap();
    write_hls(&staged, b"segment");
    write_hls(&published, b"segment");
    let inventory =
        staging::inspect_package(&staged, Path::new("master.m3u8"), deadline()).unwrap();
    let task = task(&published);
    let identity = identity_for(&task);
    let cache = ArtifactStore::new(temp.path().join("cache"));
    let output = StagedOutput::Package(StagedPackage {
        source: staged,
        target: published.clone(),
        entrypoint: "master.m3u8".into(),
        inventory: inventory.clone(),
    });

    let stored = store(&cache, &task, &identity, &[output], deadline()).unwrap();
    assert_eq!(stored.output_records[0].content, inventory.tree);
    let hit = resume(&cache, &task, &identity, deadline())
        .unwrap()
        .unwrap();
    assert_eq!(hit.paths, vec![published.clone()]);
    assert_eq!(hit.output_records, stored.output_records);

    std::fs::write(published.join("segment-rnd_main-000000.ts"), b"changed").unwrap();
    assert!(resume(&cache, &task, &identity, deadline())
        .unwrap()
        .is_none());
    assert!(cache.get(&identity.key).unwrap().is_none());
}

fn task(root: &Path) -> BackendTask {
    let paths = BackendPackagePaths {
        playlist_pattern: "rendition-%v.m3u8".into(),
        segment_pattern: "segment-%v-%06d.ts".into(),
    };
    BackendTask {
        deliverable_id: veac_ir::DeliverableId::new("dlv_package_cache").unwrap(),
        phase: BackendPhase::Single,
        product: BackendProduct::HlsVod,
        output: BackendOutput::Package {
            root: root.to_path_buf(),
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

fn write_hls(root: &Path, segment: &[u8]) {
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
    std::fs::write(root.join("segment-rnd_main-000000.ts"), segment).unwrap();
}
