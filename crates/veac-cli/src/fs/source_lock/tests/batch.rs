use super::super::{SourceModuleGuard, SourceModuleReplacement};
use super::SourceGraphLock;

#[test]
fn batch_commit_replaces_every_module_and_preserves_guards() {
    let fixture = Fixture::new();
    let lock = SourceGraphLock::acquire(fixture.root()).unwrap();
    commit_modules(
        &lock,
        fixture.root(),
        &fixture.replacements(),
        &[SourceModuleGuard {
            module: "guard.veac",
            expected: "guard",
        }],
    )
    .unwrap();
    fixture.assert_sources("after-main", "after-brand", "guard");
}

#[test]
fn empty_batch_commit_is_rejected() {
    let fixture = Fixture::new();
    let lock = SourceGraphLock::acquire(fixture.root()).unwrap();
    let error = commit_modules(&lock, fixture.root(), &[], &[]).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "SOURCE_EDIT_REJECTED");
    fixture.assert_sources("before-main", "before-brand", "guard");
}

#[test]
fn stale_later_module_prevents_any_batch_publication() {
    let fixture = Fixture::new();
    std::fs::write(fixture.brand(), "external").unwrap();
    let lock = SourceGraphLock::acquire(fixture.root()).unwrap();
    let error = commit_modules(&lock, fixture.root(), &fixture.replacements(), &[]).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    fixture.assert_sources("before-main", "external", "guard");
}

#[test]
fn duplicate_change_or_guard_module_is_rejected_without_writing() {
    let fixture = Fixture::new();
    let lock = SourceGraphLock::acquire(fixture.root()).unwrap();
    let duplicate = [
        SourceModuleReplacement {
            module: "main.veac",
            expected: "before-main",
            replacement: "first",
        },
        SourceModuleReplacement {
            module: "main.veac",
            expected: "before-main",
            replacement: "second",
        },
    ];
    let error = commit_modules(&lock, fixture.root(), &duplicate, &[]).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "SOURCE_EDIT_REJECTED");

    let guard = SourceModuleGuard {
        module: "main.veac",
        expected: "before-main",
    };
    let error =
        commit_modules(&lock, fixture.root(), &fixture.replacements(), &[guard]).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "SOURCE_EDIT_REJECTED");
    fixture.assert_sources("before-main", "before-brand", "guard");
}

#[test]
fn stale_guard_prevents_all_changed_modules_from_publishing() {
    let fixture = Fixture::new();
    std::fs::write(fixture.guard(), "changed").unwrap();
    let lock = SourceGraphLock::acquire(fixture.root()).unwrap();
    let error = commit_modules(
        &lock,
        fixture.root(),
        &fixture.replacements(),
        &[SourceModuleGuard {
            module: "guard.veac",
            expected: "guard",
        }],
    )
    .unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    fixture.assert_sources("before-main", "before-brand", "changed");
}

fn commit_modules(
    lock: &SourceGraphLock,
    root: &std::path::Path,
    replacements: &[SourceModuleReplacement<'_>],
    guards: &[SourceModuleGuard<'_>],
) -> crate::error::CliResult {
    lock.commit_modules_with(root, replacements, guards, || Ok(()), || Ok(()))
}

struct Fixture {
    _temp: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("main.veac"), "before-main").unwrap();
        std::fs::write(temp.path().join("brand.veac"), "before-brand").unwrap();
        std::fs::write(temp.path().join("guard.veac"), "guard").unwrap();
        Self { _temp: temp }
    }

    fn root(&self) -> &std::path::Path {
        self._temp.path()
    }

    fn brand(&self) -> std::path::PathBuf {
        self.root().join("brand.veac")
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
                module: "brand.veac",
                expected: "before-brand",
                replacement: "after-brand",
            },
        ]
    }

    fn assert_sources(&self, main: &str, brand: &str, guard: &str) {
        assert_eq!(
            std::fs::read_to_string(self.root().join("main.veac")).unwrap(),
            main
        );
        assert_eq!(std::fs::read_to_string(self.brand()).unwrap(), brand);
        assert_eq!(std::fs::read_to_string(self.guard()).unwrap(), guard);
    }
}
