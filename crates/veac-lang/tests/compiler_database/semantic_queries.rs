use std::collections::BTreeMap;

use super::{entry, support, CompilerDatabase, Loader};

#[test]
fn function_queries_cache_hir_and_verified_core() {
    let database = CompilerDatabase::default();
    let source = support::project_with("fn duration() -> time { 750ms }", "duration()");
    database.prepare_source(&source).unwrap();
    let first = database.statistics();
    assert!(first.hir_insertions >= 1);
    assert!(first.core_insertions >= 1);

    database.prepare_source(&source).unwrap();
    let second = database.statistics();
    assert!(second.hir_hits > first.hir_hits);
    assert!(second.core_hits > first.core_hits);
}

#[test]
fn static_constant_contexts_bypass_semantic_caches() {
    let source = support::project_with("const time base = 250ms;", "base");
    let database = CompilerDatabase::default();
    database.prepare_source(&source).unwrap();
    database.prepare_source(&source).unwrap();
    let stats = database.statistics();
    assert!(stats.hir_bypasses >= 2);
    assert!(stats.core_bypasses >= 2);
}

#[test]
fn nominal_alias_changes_cannot_reuse_stale_semantic_queries() {
    let loader = Loader {
        sources: BTreeMap::from([(
            "types.veac".into(),
            "module { export struct Timing { duration: time, } }".into(),
        )]),
    };
    let declarations = concat!(
        "import \"./types.veac\" as timing; ",
        "fn duration() -> time { timing.Timing { duration: 1s, }.duration }"
    );
    let first = entry(declarations, "duration()");
    let database = CompilerDatabase::default();
    database
        .prepare_with_loader(first.clone(), &loader)
        .unwrap();

    let changed = entry(
        &declarations.replace("as timing", "as renamed"),
        "duration()",
    );
    let cached = database
        .prepare_with_loader(changed.clone(), &loader)
        .unwrap_err();
    let clean = CompilerDatabase::default()
        .prepare_with_loader(changed, &loader)
        .unwrap_err();
    assert_eq!(cached, clean);
    assert!(cached.as_slice()[0]
        .message
        .contains("unknown nominal type"));
}
