use std::path::{Path, PathBuf};

use super::super::directory::{Directory, EntryState};
use super::super::{journal, StagedFile};
use super::{file_outputs, load_journal, prepare_journal};

#[test]
fn journal_load_rejects_non_regular_and_oversized_files() {
    let temp = tempfile::tempdir().unwrap();
    let directory_stage = stage(temp.path(), "directory-journal");
    std::fs::create_dir(directory_stage.join(journal::JOURNAL_NAME)).unwrap();
    assert_bounded_error(load_journal(&directory_stage).unwrap_err());

    let oversized_stage = stage(temp.path(), "oversized-journal");
    let journal_file = std::fs::File::create(oversized_stage.join(journal::JOURNAL_NAME)).unwrap();
    journal_file.set_len(1024 * 1024 + 1).unwrap();
    assert_bounded_error(load_journal(&oversized_stage).unwrap_err());
}

#[cfg(unix)]
#[test]
fn journal_load_refuses_to_follow_a_symlink() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path(), "symlink-journal");
    let outside = temp.path().join("outside.json");
    std::fs::write(&outside, valid_journal("output")).unwrap();
    symlink(&outside, staging.join(journal::JOURNAL_NAME)).unwrap();

    assert_bounded_error(load_journal(&staging).unwrap_err());
}

#[test]
fn journal_load_rejects_empty_entries_and_parent_components() {
    let temp = tempfile::tempdir().unwrap();
    let empty = stage(temp.path(), "empty-journal");
    std::fs::write(
        empty.join(journal::JOURNAL_NAME),
        br#"{"entries":[],"schema_version":3,"state":"prepared"}"#,
    )
    .unwrap();
    let error = load_journal(&empty).unwrap_err();
    assert!(error.message.contains("unsupported or empty"));

    let parent = stage(temp.path(), "parent-component");
    std::fs::write(parent.join(journal::JOURNAL_NAME), valid_journal("..")).unwrap();
    let error = load_journal(&parent).unwrap_err();
    assert!(error.message.contains("single-component"));

    let malformed = stage(temp.path(), "malformed-json");
    std::fs::write(malformed.join(journal::JOURNAL_NAME), b"{").unwrap();
    let error = load_journal(&malformed).unwrap_err();
    assert!(error.message.contains("invalid render commit journal"));
}

#[test]
fn journal_prepare_rejects_a_target_without_a_file_name() {
    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path(), "root-target");
    let file = staged(&staging, "source", PathBuf::from("/"));

    let error = prepare_journal(&staging, &[file], &[]).unwrap_err();

    assert!(error.message.contains("path has no file name"));
}

#[cfg(unix)]
#[test]
fn journal_prepare_rejects_a_non_utf8_target() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path(), "non-utf8-target");
    let target = temp.path().join(OsString::from_vec(vec![0xff]));
    let file = staged(&staging, "source", target);

    let error = prepare_journal(&staging, &[file], &[]).unwrap_err();

    assert!(error.message.contains("paths must be valid UTF-8"));
}

#[test]
fn journal_prepare_rejects_targets_in_different_directories() {
    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path(), "split-targets");
    let left = temp.path().join("left");
    let right = temp.path().join("right");
    std::fs::create_dir(&left).unwrap();
    std::fs::create_dir(&right).unwrap();
    let files = [
        staged(&staging, "left-source", left.join("output")),
        staged(&staging, "right-source", right.join("output")),
    ];

    let error = prepare_journal(&staging, &files, &[]).unwrap_err();

    assert!(error.message.contains("targets must share one directory"));
    assert!(!staging.join(journal::JOURNAL_NAME).exists());
}

#[test]
fn journal_prepare_preserves_an_existing_temporary_file() {
    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path(), "existing-temporary");
    let file = staged(&staging, "source", temp.path().join("output"));
    let temporary = staging.join(".veac-commit.tmp");
    std::fs::write(&temporary, b"untrusted").unwrap();

    let error = prepare_journal(&staging, &[file], &[]).unwrap_err();

    assert!(error.message.contains("journal I/O failed"));
    assert_eq!(std::fs::read(temporary).unwrap(), b"untrusted");
    assert!(!staging.join(journal::JOURNAL_NAME).exists());
}

#[test]
fn journal_persistence_stays_bound_to_the_open_stage_directory() {
    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path(), "bound-journal");
    let moved = temp.path().join("moved-stage");
    let file = staged(&staging, "source", temp.path().join("output"));
    let descriptor = Directory::open(&staging).unwrap();
    let output = Directory::open(temp.path()).unwrap();
    let EntryState::Regular(identity) = descriptor.state("source").unwrap() else {
        panic!("fixture source is regular");
    };
    std::fs::rename(&staging, &moved).unwrap();
    std::fs::create_dir(&staging).unwrap();
    std::fs::write(staging.join("sentinel"), b"foreign").unwrap();

    let outputs = file_outputs(&[file]);
    journal::prepare(&descriptor, &output, &outputs, &[identity], &[]).unwrap();

    assert!(moved.join(journal::JOURNAL_NAME).is_file());
    assert!(!staging.join(journal::JOURNAL_NAME).exists());
    assert_eq!(std::fs::read(staging.join("sentinel")).unwrap(), b"foreign");
}

fn assert_bounded_error(error: crate::RuntimeError) {
    assert!(error.message.contains("bounded regular non-symlink"));
}

fn valid_journal(target: &str) -> Vec<u8> {
    format!(
        r#"{{"entries":[{{"original":null,"source":"source","source_identity":{{"device":1,"inode":1,"node_type":"regular","size_bytes":1}},"target":"{target}"}}],"schema_version":3,"state":"prepared"}}"#
    )
    .into_bytes()
}

fn stage(parent: &Path, name: &str) -> PathBuf {
    let path = parent.join(format!(".veac-stage-{name}"));
    std::fs::create_dir(&path).unwrap();
    path
}

fn staged(staging: &Path, source: &str, target: PathBuf) -> StagedFile {
    let source = staging.join(source);
    std::fs::write(&source, b"new").unwrap();
    StagedFile {
        source,
        target,
        allow_empty: false,
    }
}
