use crate::program::expand::definition::Budget as DefinitionBudget;
use crate::program::model::{ComponentCatalog, Scope, SurfaceFile};
use crate::program::Diagnostic;

use super::{resolve, source};

#[test]
fn component_logical_storage_has_an_exact_deterministic_boundary() {
    let file = source(
        r#"component sequence leaf { body {} }
component sequence card {
  param text title default "hello";
  slot visual media;
  instance sequence @nested from leaf {}
  body {}
}"#,
    );
    let charged = resolve_with_limit(&file, usize::MAX).unwrap();
    assert!(charged > file.source.len());
    assert_eq!(resolve_with_limit(&file, charged).unwrap(), charged);
    let error = resolve_with_limit(&file, charged - 1).unwrap_err();
    assert_eq!(error.code, "PROGRAM_RETAINED_SCOPE_BUDGET");
}

#[test]
fn component_semantics_precede_retained_storage_budget() {
    let file = source(
        r#"component sequence recursive {
  instance sequence @again from recursive {}
  body {}
}"#,
    );
    let error = resolve_with_limit(&file, 0).unwrap_err();
    assert_eq!(error.code, "PROGRAM_COMPONENT_CYCLE");
}

fn resolve_with_limit(file: &SurfaceFile, limit: usize) -> Result<usize, Diagnostic> {
    let mut scope = Scope::default();
    let mut catalog = ComponentCatalog::new();
    let mut definitions = DefinitionBudget::default();
    let mut retained = crate::program::resolve::retained::Budget { bytes: 0, limit };
    resolve(
        file,
        &mut scope,
        &mut catalog,
        &mut definitions,
        &mut retained,
    )?;
    Ok(retained.bytes)
}
