use std::collections::BTreeMap;

use crate::authoring::Span;
use crate::program::loader::MemoryLoader;

use super::super::*;

const MODULE: &str = r#"module {
  export component sequence leaf { body {} }
}"#;

#[test]
fn imported_component_capture_storage_has_one_resolver_wide_exact_boundary() {
    let source = project(
        r#"import "./shared.veac" as shared;
component sequence card {
  instance sequence @child from shared.leaf {}
  body {}
}"#,
    );
    let charged = resolve_with_limit(&source, usize::MAX).unwrap();
    assert_eq!(resolve_with_limit(&source, charged).unwrap(), charged);
    let error = resolve_with_limit(&source, charged - 1).unwrap_err();
    assert_eq!(error.code, "PROGRAM_RETAINED_SCOPE_BUDGET");
}

fn resolve_with_limit(source: &str, limit: usize) -> Result<usize, Diagnostic> {
    let file = parser::parse("main.veac", source).unwrap();
    let mut source_budget = SourceBudget::default();
    source_budget
        .add("main.veac", source, Span::default())
        .unwrap();
    let loader = MemoryLoader::new(BTreeMap::from([(
        "shared.veac".to_owned(),
        MODULE.to_owned(),
    )]));
    let mut resolver = Resolver::new(&loader, source_budget);
    resolver.retained = retained::Budget { bytes: 0, limit };
    resolver.sources.insert("main.veac".into(), source.into());
    resolver.active.push("main.veac".into());
    let result = resolver.scope(&file, false);
    resolver.active.pop();
    result?;
    Ok(resolver.retained.bytes)
}

fn project(declarations: &str) -> String {
    format!(
        r#"{declarations}
project captures {{
  settings {{
    timebase 1/1000; canvas 1px by 1px;
    frame-rate 1fps; sample-rate 8000hz;
  }}
  entry sequence main; sequence main {{}}
}}"#
    )
}
