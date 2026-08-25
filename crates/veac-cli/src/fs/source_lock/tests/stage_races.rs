use rustix::fs::Mode;

use super::super::{path, stage::Staged};
use super::SourceGraphLock;

#[test]
fn publication_rejects_tampered_stage_bytes() {
    let fixture = fixture();
    let staged = Staged::create(
        &fixture.parent.descriptor,
        Mode::RUSR | Mode::WUSR,
        b"after",
        &fixture.source,
    )
    .unwrap();
    let expected = fixture.expected();
    let stage = stage_path(fixture.temp.path());
    std::fs::write(stage, "tampered").unwrap();

    let error = staged
        .publish(&fixture.parent, &expected, &fixture.source)
        .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "WRITE_FAILED");
    assert_eq!(std::fs::read_to_string(fixture.source).unwrap(), "before");
}

#[test]
fn publication_rejects_a_hard_link_to_the_stage() {
    let fixture = fixture();
    let staged = Staged::create(
        &fixture.parent.descriptor,
        Mode::RUSR | Mode::WUSR,
        b"after",
        &fixture.source,
    )
    .unwrap();
    let expected = fixture.expected();
    let stage = stage_path(fixture.temp.path());
    let alias = fixture.temp.path().join("stage-alias");
    std::fs::hard_link(&stage, &alias).unwrap();

    let error = staged
        .publish(&fixture.parent, &expected, &fixture.source)
        .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "WRITE_FAILED");
    assert_eq!(std::fs::read_to_string(&fixture.source).unwrap(), "before");
    std::fs::remove_file(alias).unwrap();
    std::fs::remove_file(stage).unwrap();
}

#[test]
fn publication_rejects_a_removed_stage_path() {
    let fixture = fixture();
    let staged = Staged::create(
        &fixture.parent.descriptor,
        Mode::RUSR | Mode::WUSR,
        b"after",
        &fixture.source,
    )
    .unwrap();
    let expected = fixture.expected();
    std::fs::remove_file(stage_path(fixture.temp.path())).unwrap();

    let error = staged
        .publish(&fixture.parent, &expected, &fixture.source)
        .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "WRITE_FAILED");
    assert!(error.to_string().contains("inspect stage path"));
    assert_eq!(std::fs::read_to_string(fixture.source).unwrap(), "before");
}

struct Fixture {
    temp: tempfile::TempDir,
    source: std::path::PathBuf,
    parent: path::Parent,
}

impl Fixture {
    fn expected(&self) -> super::super::stage::ExpectedTarget {
        let target = path::read_target(&self.parent, &self.source, 64).unwrap();
        super::super::stage::ExpectedTarget::from_target(&target)
    }
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("main.veac");
    std::fs::write(&source, "before").unwrap();
    let lock = SourceGraphLock::acquire(temp.path()).unwrap();
    let parent = path::resolve(&lock.directory, "main.veac", &source).unwrap();
    drop(lock);
    Fixture {
        temp,
        source,
        parent,
    }
}

fn stage_path(root: &std::path::Path) -> std::path::PathBuf {
    std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(".veac-source-stage-")
        })
        .unwrap()
}
