use super::tests::Fixture;
use super::{prepare, publish, stage};
use crate::error::CliError;
use crate::fs::SourceGraphLock;

#[test]
fn validation_failure_rolls_back_every_module() {
    let fixture = Fixture::new();
    let lock = SourceGraphLock::acquire(fixture.root()).unwrap();
    let values = fixture.replacements();
    let prepared = prepare(&lock, fixture.root(), &values).unwrap();
    let replacements = stage(&prepared, &values, false).unwrap();
    let rollbacks = stage(&prepared, &values, true).unwrap();

    let error = publish::run(
        publish::Context::new(&lock, fixture.root(), &prepared, &[], &[]),
        replacements,
        rollbacks,
        || Err(CliError::new("SOURCE_CHANGED", "stale graph")),
    )
    .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    fixture.assert_sources("before-main", "before-brand");
}

#[test]
fn final_guard_verification_rolls_back_every_module() {
    let fixture = Fixture::new();
    let lock = SourceGraphLock::acquire(fixture.root()).unwrap();
    let values = fixture.replacements();
    let guards = [super::SourceModuleGuard {
        module: "guard.veac",
        expected: "guard",
    }];
    let guarded = super::guard::prepare(&lock, fixture.root(), &guards).unwrap();
    let prepared = prepare(&lock, fixture.root(), &values).unwrap();
    let replacements = stage(&prepared, &values, false).unwrap();
    let rollbacks = stage(&prepared, &values, true).unwrap();

    let error = publish::run(
        publish::Context::new(&lock, fixture.root(), &prepared, &guarded, &guards),
        replacements,
        rollbacks,
        || {
            std::fs::write(fixture.guard(), "drifted").unwrap();
            Ok(())
        },
    )
    .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    fixture.assert_sources("before-main", "before-brand");
}

#[test]
fn rollback_cas_preserves_an_external_target_replacement() {
    let fixture = Fixture::new();
    let lock = SourceGraphLock::acquire(fixture.root()).unwrap();
    let values = fixture.replacements();
    let prepared = prepare(&lock, fixture.root(), &values).unwrap();
    let replacements = stage(&prepared, &values, false).unwrap();
    let rollbacks = stage(&prepared, &values, true).unwrap();
    let source = fixture.root().join("main.veac");

    let error = publish::run(
        publish::Context::new(&lock, fixture.root(), &prepared, &[], &[]),
        replacements,
        rollbacks,
        || {
            std::fs::rename(&source, fixture.root().join("published-main.veac")).unwrap();
            std::fs::write(&source, "external-main").unwrap();
            Err(CliError::new("SOURCE_CHANGED", "force rollback"))
        },
    )
    .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "WRITE_COMMIT_UNCERTAIN");
    fixture.assert_sources("external-main", "before-brand");
}
