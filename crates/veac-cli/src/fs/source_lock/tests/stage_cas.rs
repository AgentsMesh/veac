use rustix::fs::Mode;

use super::super::{path, stage::ExpectedTarget, stage::Staged};
use super::SourceGraphLock;

#[test]
fn exchange_publish_replaces_the_expected_target() {
    let fixture = Fixture::new();
    let (staged, expected) = fixture.stage_and_expected();

    staged
        .publish(&fixture.parent, &expected, &fixture.source)
        .unwrap();

    assert_eq!(std::fs::read_to_string(&fixture.source).unwrap(), "after");
    assert_eq!(stage_count(fixture.temp.path()), 0);
}

#[test]
fn exchange_cas_preserves_a_replaced_target() {
    let fixture = Fixture::new();
    let (staged, expected) = fixture.stage_and_expected();
    std::fs::rename(&fixture.source, fixture.temp.path().join("original.veac")).unwrap();
    std::fs::write(&fixture.source, "external").unwrap();

    let error = staged
        .publish(&fixture.parent, &expected, &fixture.source)
        .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    assert_eq!(
        std::fs::read_to_string(&fixture.source).unwrap(),
        "external"
    );
}

#[test]
fn exchange_cas_preserves_an_in_place_target_change() {
    let fixture = Fixture::new();
    let (staged, expected) = fixture.stage_and_expected();
    std::fs::write(&fixture.source, "external").unwrap();

    let error = staged
        .publish(&fixture.parent, &expected, &fixture.source)
        .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    assert_eq!(
        std::fs::read_to_string(&fixture.source).unwrap(),
        "external"
    );
}

struct Fixture {
    temp: tempfile::TempDir,
    source: std::path::PathBuf,
    parent: path::Parent,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("main.veac");
        std::fs::write(&source, "before").unwrap();
        let lock = SourceGraphLock::acquire(temp.path()).unwrap();
        let parent = path::resolve(&lock.directory, "main.veac", &source).unwrap();
        drop(lock);
        Self {
            temp,
            source,
            parent,
        }
    }

    fn stage_and_expected(&self) -> (Staged<'_>, ExpectedTarget) {
        let target = path::read_target(&self.parent, &self.source, 64).unwrap();
        let expected = ExpectedTarget::from_target(&target);
        let staged = Staged::create(
            &self.parent.descriptor,
            Mode::RUSR | Mode::WUSR,
            b"after",
            &self.source,
        )
        .unwrap();
        (staged, expected)
    }
}

fn stage_count(root: &std::path::Path) -> usize {
    std::fs::read_dir(root)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".veac-source-stage-")
        })
        .count()
}
