use std::path::{Path, PathBuf};

use super::super::directory::{Directory, EntryState};
use super::super::{journal, package, StagedOutput, StagedPackage};
use super::{commit_journal, deadline, recover};

#[test]
fn prepared_package_recovery_restores_the_complete_old_tree() {
    let fixture = partial_package_commit();
    recover(fixture.root.path()).unwrap();
    assert_eq!(
        std::fs::read(fixture.target.join("old.ts")).unwrap(),
        b"old"
    );
    assert!(!fixture.target.join("master.m3u8").exists());
    assert!(!fixture.staging.exists());
}

#[test]
fn committed_package_recovery_keeps_new_tree_and_removes_backup() {
    let mut fixture = partial_package_commit();
    commit_journal(&fixture.staging, &mut fixture.journal).unwrap();
    recover(fixture.root.path()).unwrap();
    assert!(fixture.target.join("master.m3u8").is_file());
    assert!(fixture.target.join("segment-rnd_main-000000.ts").is_file());
    assert!(!fixture.target.join("old.ts").exists());
    assert!(!fixture.staging.exists());
}

struct Fixture {
    root: tempfile::TempDir,
    staging: PathBuf,
    target: PathBuf,
    journal: journal::Journal,
}

fn partial_package_commit() -> Fixture {
    let root = tempfile::tempdir().unwrap();
    let staging = root.path().join(".veac-stage-package-recovery");
    let source = staging.join("package");
    let target = root.path().join("stream");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::create_dir(&target).unwrap();
    std::fs::create_dir(staging.join("backups")).unwrap();
    std::fs::write(target.join("old.ts"), b"old").unwrap();
    write_hls(&source);
    let inventory = package::inspect_hls(&source, Path::new("master.m3u8"), deadline()).unwrap();
    let output_value = StagedOutput::Package(StagedPackage {
        source: source.clone(),
        target: target.clone(),
        entrypoint: "master.m3u8".into(),
        inventory,
    });
    let stage = Directory::open(&staging).unwrap();
    let output = Directory::open(root.path()).unwrap();
    let EntryState::Directory(source_identity) = stage.state("package").unwrap() else {
        panic!("fixture package is a directory");
    };
    let journal =
        journal::prepare(&stage, &output, &[output_value], &[source_identity], &[]).unwrap();
    std::fs::rename(&target, staging.join("backups/0")).unwrap();
    std::fs::rename(&source, &target).unwrap();
    drop(stage);
    drop(output);
    Fixture {
        root,
        staging,
        target,
        journal,
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
    std::fs::write(root.join("segment-rnd_main-000000.ts"), b"new").unwrap();
}
