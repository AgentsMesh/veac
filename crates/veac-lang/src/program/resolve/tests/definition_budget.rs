use std::collections::BTreeMap;

use crate::authoring::Span;
use crate::program::expand::definition::Budget as DefinitionBudget;
use crate::program::loader::MemoryLoader;

use super::super::*;

const MODULE: &str = r#"module {
  export preset text-style title {
    font family "Arial"; size 12px; fill #ffffffff;
  }
}"#;

#[test]
fn imported_preset_and_entry_component_share_one_definition_budget() {
    let source = project("import \"./style.veac\" as style;\ncomponent sequence card { body {} }");
    resolve_with_units(&source, 3).unwrap();
    let error = resolve_with_units(&source, 2).unwrap_err();
    assert_eq!(error.code, "PROGRAM_DEFINITION_BUDGET");
    assert!(error.message.contains("validation units"));
}

#[test]
fn invalid_preset_semantics_precede_an_exhausted_definition_budget() {
    let source = project("preset text-style broken { impossible syntax; }");
    let error = resolve_with_units(&source, 0).unwrap_err();
    assert_eq!(error.code, "PROGRAM_PRESET_DEFINITION");
}

fn resolve_with_units(source: &str, units: usize) -> Result<Scope, Diagnostic> {
    let file = parser::parse("main.veac", source).unwrap();
    let mut source_budget = SourceBudget::default();
    source_budget
        .add("main.veac", source, Span::default())
        .unwrap();
    let loader = MemoryLoader::new(BTreeMap::from([(
        "style.veac".to_owned(),
        MODULE.to_owned(),
    )]));
    let mut resolver = Resolver::new(&loader, source_budget);
    resolver.definitions = DefinitionBudget::with_limits(units, usize::MAX, usize::MAX);
    resolver.sources.insert("main.veac".into(), source.into());
    resolver.active.push("main.veac".into());
    let result = resolver.scope(&file, false);
    resolver.active.pop();
    result
}

fn project(declarations: &str) -> String {
    format!(
        r#"{declarations}
project definitions {{
  settings {{
    timebase 1/1000; canvas 1px by 1px;
    frame-rate 1fps; sample-rate 8000hz;
  }}
  entry sequence main; sequence main {{}}
}}"#
    )
}
