use super::{classify_source_unit, SourceUnitKind};

#[test]
fn distinguishes_executable_entries_and_modules() {
    assert_eq!(
        classify_source_unit(
            "main.veac",
            "fn main(context: Context) -> Project { context }",
        )
        .unwrap(),
        SourceUnitKind::Entry
    );
    assert_eq!(
        classify_source_unit("brand.veac", "module {}").unwrap(),
        SourceUnitKind::Module
    );
}

#[test]
fn rejects_unparseable_units() {
    let error = classify_source_unit("bad.veac", "not valid VEAC").unwrap_err();
    assert!(!error.as_slice().is_empty());
}
