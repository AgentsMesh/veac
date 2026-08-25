use super::tests::Fixture;
use super::{prepare, publish, stage};
use crate::error::CliError;
use crate::fs::SourceGraphLock;

#[test]
fn second_target_cas_failure_rolls_back_first_and_preserves_external_bytes() {
    let fixture = Fixture::new();
    let lock = SourceGraphLock::acquire(fixture.root()).unwrap();
    let values = fixture.replacements();
    let prepared = prepare(&lock, fixture.root(), &values).unwrap();
    let replacements = stage(&prepared, &values, false).unwrap();
    let rollbacks = stage(&prepared, &values, true).unwrap();
    let brand = fixture.parts().join("brand.veac");
    std::fs::rename(&brand, fixture.parts().join("original-brand.veac")).unwrap();
    std::fs::write(&brand, "external-brand").unwrap();

    let error = publish::run(
        publish::Context::new(&lock, fixture.root(), &prepared, &[], &[]),
        replacements,
        rollbacks,
        || Ok(()),
    )
    .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    fixture.assert_sources("before-main", "external-brand");
}

#[test]
fn rollback_continues_after_preserving_an_external_later_target() {
    let fixture = Fixture::new();
    let lock = SourceGraphLock::acquire(fixture.root()).unwrap();
    let values = fixture.replacements();
    let prepared = prepare(&lock, fixture.root(), &values).unwrap();
    let replacements = stage(&prepared, &values, false).unwrap();
    let rollbacks = stage(&prepared, &values, true).unwrap();
    let brand = fixture.parts().join("brand.veac");

    let error = publish::run(
        publish::Context::new(&lock, fixture.root(), &prepared, &[], &[]),
        replacements,
        rollbacks,
        || {
            std::fs::rename(&brand, fixture.parts().join("published-brand.veac")).unwrap();
            std::fs::write(&brand, "external-brand").unwrap();
            Err(CliError::new("SOURCE_CHANGED", "force rollback"))
        },
    )
    .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "WRITE_COMMIT_UNCERTAIN");
    fixture.assert_sources("before-main", "external-brand");
}
