use std::path::Path;

use super::*;

#[path = "tests/cas.rs"]
mod cas;
#[path = "tests/post_publish.rs"]
mod post_publish;

#[test]
fn guard_verification_rejects_byte_and_same_content_inode_replacement() {
    for same_content in [false, true] {
        let fixture = Fixture::new();
        let lock = SourceGraphLock::acquire(fixture.root()).unwrap();
        let values = [SourceModuleGuard {
            module: "guard.veac",
            expected: "guard",
        }];
        let prepared = guard::prepare(&lock, fixture.root(), &values).unwrap();
        if same_content {
            std::fs::rename(fixture.guard(), fixture.root().join("old-guard.veac")).unwrap();
            std::fs::write(fixture.guard(), "guard").unwrap();
        } else {
            std::fs::write(fixture.guard(), "other").unwrap();
        }
        let error = guard::verify(&lock, &prepared, &values).unwrap_err();
        assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    }
}

#[test]
fn second_publish_failure_rolls_back_the_first_module() {
    let fixture = Fixture::new();
    let lock = SourceGraphLock::acquire(fixture.root()).unwrap();
    let values = fixture.replacements();
    let prepared = prepare(&lock, fixture.root(), &values).unwrap();
    let replacements = stage(&prepared, &values, false).unwrap();
    let rollbacks = stage(&prepared, &values, true).unwrap();
    remove_stage_with(&fixture.parts(), b"after-brand");

    let error = publish::run(
        publish::Context::new(&lock, fixture.root(), &prepared, &[], &[]),
        replacements,
        rollbacks,
        || Ok(()),
    )
    .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "WRITE_FAILED");
    fixture.assert_sources("before-main", "before-brand");
}

#[test]
fn rollback_failure_reports_commit_uncertainty() {
    let fixture = Fixture::new();
    let lock = SourceGraphLock::acquire(fixture.root()).unwrap();
    let values = fixture.replacements();
    let prepared = prepare(&lock, fixture.root(), &values).unwrap();
    let replacements = stage(&prepared, &values, false).unwrap();
    let rollbacks = stage(&prepared, &values, true).unwrap();
    remove_stage_with(&fixture.parts(), b"after-brand");
    remove_stage_with(fixture.root(), b"before-main");

    let error = publish::run(
        publish::Context::new(&lock, fixture.root(), &prepared, &[], &[]),
        replacements,
        rollbacks,
        || Ok(()),
    )
    .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "WRITE_COMMIT_UNCERTAIN");
    fixture.assert_sources("after-main", "before-brand");
}

#[test]
fn final_check_rejects_same_content_target_replacement() {
    let fixture = Fixture::new();
    let lock = SourceGraphLock::acquire(fixture.root()).unwrap();
    let values = fixture.replacements();
    let prepared = prepare(&lock, fixture.root(), &values).unwrap();
    let source = fixture.root().join("main.veac");
    std::fs::rename(&source, fixture.root().join("old-main.veac")).unwrap();
    std::fs::write(&source, "before-main").unwrap();
    let error = final_check(&lock, fixture.root(), &prepared, &values).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
}

#[test]
fn guard_change_after_staging_rolls_back_every_published_module() {
    let fixture = Fixture::new();
    let lock = SourceGraphLock::acquire(fixture.root()).unwrap();
    let values = fixture.replacements();
    let guards = [SourceModuleGuard {
        module: "guard.veac",
        expected: "guard",
    }];
    let guarded = guard::prepare(&lock, fixture.root(), &guards).unwrap();
    let prepared = prepare(&lock, fixture.root(), &values).unwrap();
    let replacements = stage(&prepared, &values, false).unwrap();
    let rollbacks = stage(&prepared, &values, true).unwrap();
    std::fs::write(fixture.guard(), "other").unwrap();
    let error = publish::run(
        publish::Context::new(&lock, fixture.root(), &prepared, &guarded, &guards),
        replacements,
        rollbacks,
        || Ok(()),
    )
    .unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    fixture.assert_sources("before-main", "before-brand");
}

struct Fixture {
    _temp: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir(temp.path().join("parts")).unwrap();
        std::fs::write(temp.path().join("main.veac"), "before-main").unwrap();
        std::fs::write(temp.path().join("parts/brand.veac"), "before-brand").unwrap();
        std::fs::write(temp.path().join("guard.veac"), "guard").unwrap();
        Self { _temp: temp }
    }

    fn root(&self) -> &Path {
        self._temp.path()
    }

    fn parts(&self) -> std::path::PathBuf {
        self.root().join("parts")
    }

    fn guard(&self) -> std::path::PathBuf {
        self.root().join("guard.veac")
    }

    fn replacements(&self) -> [SourceModuleReplacement<'static>; 2] {
        [
            SourceModuleReplacement {
                module: "main.veac",
                expected: "before-main",
                replacement: "after-main",
            },
            SourceModuleReplacement {
                module: "parts/brand.veac",
                expected: "before-brand",
                replacement: "after-brand",
            },
        ]
    }

    fn assert_sources(&self, main: &str, brand: &str) {
        assert_eq!(
            std::fs::read_to_string(self.root().join("main.veac")).unwrap(),
            main
        );
        assert_eq!(
            std::fs::read_to_string(self.root().join("parts/brand.veac")).unwrap(),
            brand
        );
    }
}

fn remove_stage_with(directory: &Path, content: &[u8]) {
    let path = std::fs::read_dir(directory)
        .unwrap()
        .map(|value| value.unwrap().path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(".veac-source-stage-")
                && std::fs::read(path).ok().as_deref() == Some(content)
        })
        .expect("matching stage");
    std::fs::remove_file(path).unwrap();
}
