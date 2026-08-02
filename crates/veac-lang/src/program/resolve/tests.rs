use std::collections::BTreeMap;
use std::sync::Arc;

use crate::authoring::Span;
use crate::program::loader::{LoadedSource, MemoryLoader};

use super::*;

mod component_capture_budget;
mod definition_budget;

const MODULE: &str = r##"module {
  export const text payload = "shared-payload";
  export preset text-style title {
    font family "Arial"; size 12px; fill #ffffffff;
  }
}"##;

#[test]
fn repeated_aliases_share_value_preset_and_scope_storage() {
    let resolution = entry(
        LoadedSource {
            id: "main.veac".to_owned(),
            source: project(imports(true)),
        },
        &loader(),
    )
    .unwrap();
    let values = resolution.scope.values.as_ref();
    assert!(Arc::ptr_eq(
        &values["first.payload"],
        &values["second.payload"]
    ));
    let presets = resolution.scope.presets.as_ref();
    assert!(Arc::ptr_eq(
        &presets[&(
            "first.title".into(),
            crate::program::model::PresetKind::TextStyle
        )],
        &presets[&(
            "second.title".into(),
            crate::program::model::PresetKind::TextStyle
        )]
    ));
    let cloned = resolution.scope.clone();
    assert!(Arc::ptr_eq(&resolution.scope.values, &cloned.values));
    assert!(Arc::ptr_eq(&resolution.scope.presets, &cloned.presets));
}

#[test]
fn module_cache_hits_share_scope_and_do_not_recharge_payloads() {
    let root = project("");
    let mut source_budget = SourceBudget::default();
    source_budget
        .add("main.veac", &root, Span::default())
        .unwrap();
    let loader = loader();
    let mut resolver = Resolver::new(&loader, source_budget);
    resolver.sources.insert("main.veac".into(), root);
    resolver.active.push("main.veac".into());
    let first = resolver
        .module("main.veac", "./shared.veac", Span::default())
        .unwrap();
    let charged = resolver.retained.bytes;
    let second = resolver
        .module("main.veac", "./shared.veac", Span::default())
        .unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(resolver.retained.bytes, charged);
}

#[test]
fn real_alias_path_charges_payload_once_and_has_an_exact_boundary() {
    let (_, one) = resolve_with_limit(&project(imports(false)), usize::MAX).unwrap();
    let source = project(imports(true));
    let (_, two) = resolve_with_limit(&source, usize::MAX).unwrap();
    let expected_alias = retained::ENTRY_BYTES * 2 + "second.payload".len() + "second.title".len();
    assert_eq!(two - one, expected_alias);
    assert_eq!(resolve_with_limit(&source, two).unwrap().1, two);
    let error = resolve_with_limit(&source, two - 1).unwrap_err();
    assert_eq!(error.code, "PROGRAM_RETAINED_SCOPE_BUDGET");
}

#[test]
fn semantic_errors_take_priority_over_retained_budget_errors() {
    let invalid_constant = project("const text broken = missing;");
    let error = resolve_with_limit(&invalid_constant, 0).unwrap_err();
    assert_eq!(error.code, "PROGRAM_CONST_EXPRESSION");

    let invalid_preset = project("preset text-style broken { impossible syntax; }");
    let error = resolve_with_limit(&invalid_preset, 0).unwrap_err();
    assert_eq!(error.code, "PROGRAM_PRESET_DEFINITION");

    let preset_cycle = project(
        "preset text-style first { use text-style second; }\n\
         preset text-style second { use text-style first; }",
    );
    let error = resolve_with_limit(&preset_cycle, 0).unwrap_err();
    assert_eq!(error.code, "PROGRAM_PRESET_CYCLE");

    let valid = project("const scalar retained = 1;");
    let error = resolve_with_limit(&valid, 0).unwrap_err();
    assert_eq!(error.code, "PROGRAM_RETAINED_SCOPE_BUDGET");
}

fn resolve_with_limit(source: &str, limit: usize) -> Result<(Scope, usize), Diagnostic> {
    let file = parser::parse("main.veac", source).unwrap();
    let mut source_budget = SourceBudget::default();
    source_budget
        .add("main.veac", source, Span::default())
        .unwrap();
    let loader = loader();
    let mut resolver = Resolver::new(&loader, source_budget);
    resolver.retained = retained::Budget { bytes: 0, limit };
    resolver.sources.insert("main.veac".into(), source.into());
    resolver.active.push("main.veac".into());
    let result = resolver.scope(&file, false);
    resolver.active.pop();
    let scope = result?;
    Ok((scope, resolver.retained.bytes))
}

fn loader() -> MemoryLoader {
    MemoryLoader::new(BTreeMap::from([(
        "shared.veac".to_owned(),
        MODULE.to_owned(),
    )]))
}

fn imports(twice: bool) -> &'static str {
    if twice {
        "import \"./shared.veac\" as first; import \"./shared.veac\" as second;"
    } else {
        "import \"./shared.veac\" as first;"
    }
}

fn project(declarations: &str) -> String {
    format!(
        r#"{declarations}
project retained {{
  settings {{ timebase 1/1000; canvas 1px by 1px; frame-rate 1fps; sample-rate 8000hz; }}
  entry sequence main; sequence main {{}}
}}"#
    )
}
