use super::bounded_cache::BoundedCache;
use super::*;
use crate::program::expression::{ExpressionContext, FunctionDefinition, PrimitiveType, ValueType};
use crate::program::loader::{LoadedSource, MemoryLoader};
use std::sync::Arc;

const ENTRY: &str =
    "fn main(context: Context) -> Project { project(identifier(\"p\"), project_settings(1)) }";
const MODULE: &str = "module { export fn identity(value: int) -> int { value } }";

#[test]
fn zero_byte_limit_bypasses_even_a_zero_byte_value() {
    let mut cache = BoundedCache::new(1, 0);
    let value = Arc::new(());
    let outcome = cache.insert("key", 0, Arc::clone(&value));

    assert!(outcome.bypassed);
    assert!(!outcome.inserted);
    assert!(Arc::ptr_eq(&outcome.value, &value));
    assert_eq!(cache.len(), 0);
    assert_eq!(cache.retained_bytes(), 0);
}

#[test]
fn syntax_budget_charges_the_retained_surface_not_only_source_text() {
    let byte_limit = ENTRY.len() + 1;
    let database = CompilerDatabase::with_limits(CompilerDatabaseLimits {
        syntax_source_bytes: byte_limit,
        ..Default::default()
    });
    database.parse("main.veac", ENTRY).unwrap();
    database.parse("main.veac", ENTRY).unwrap();

    let stats = database.statistics();
    assert!(byte_limit > ENTRY.len());
    assert_eq!(stats.syntax_bypasses, 2);
    assert_eq!(stats.cached_syntax_entries, 0);
    assert_eq!(stats.cached_syntax_source_bytes, 0);
}

#[test]
fn nonzero_byte_budget_evicts_the_oldest_complete_syntax_entry() {
    let second = ENTRY.replace("identifier(\"p\")", "identifier(\"long-project-name\")");
    let first_bytes = syntax_bytes("first.veac", ENTRY);
    let second_bytes = syntax_bytes("second.veac", &second);
    let database = CompilerDatabase::with_limits(CompilerDatabaseLimits {
        syntax_entries: 8,
        syntax_source_bytes: first_bytes.max(second_bytes),
        ..Default::default()
    });

    database.parse("first.veac", ENTRY).unwrap();
    database.parse("second.veac", &second).unwrap();

    let stats = database.statistics();
    assert_eq!(stats.syntax_evictions, 1);
    assert_eq!(stats.syntax_bypasses, 0);
    assert_eq!(stats.cached_syntax_entries, 1);
    assert_eq!(stats.cached_syntax_source_bytes, second_bytes);
}

#[test]
fn zero_interface_budget_recomputes_without_retaining_results() {
    let database = CompilerDatabase::with_limits(CompilerDatabaseLimits {
        interface_retained_bytes: 0,
        ..Default::default()
    });
    database
        .module_interface_shared(module(), &MemoryLoader::default())
        .unwrap();
    database
        .module_interface_shared(module(), &MemoryLoader::default())
        .unwrap();

    let stats = database.statistics();
    assert_eq!(stats.interface_bypasses, 2);
    assert_eq!(stats.cached_interface_entries, 0);
    assert_eq!(stats.cached_interface_retained_bytes, 0);
}

#[test]
fn zero_hir_budget_still_allows_core_reuse() {
    let database = CompilerDatabase::with_limits(CompilerDatabaseLimits {
        hir_retained_bytes: 0,
        ..Default::default()
    });
    compile(&database, function("one", "1")).unwrap();
    compile(&database, function("one", "1")).unwrap();

    let stats = database.statistics();
    assert_eq!(stats.hir_bypasses, 1);
    assert_eq!(stats.cached_hir_entries, 0);
    assert_eq!(stats.core_hits, 1);
    assert_eq!(stats.cached_core_entries, 1);
}

#[test]
fn zero_core_budget_relowers_a_reused_hir_batch() {
    let database = CompilerDatabase::with_limits(CompilerDatabaseLimits {
        core_retained_bytes: 0,
        ..Default::default()
    });
    compile(&database, function("one", "1")).unwrap();
    compile(&database, function("one", "1")).unwrap();

    let stats = database.statistics();
    assert_eq!(stats.hir_hits, 1);
    assert_eq!(stats.cached_hir_entries, 1);
    assert_eq!(stats.core_bypasses, 2);
    assert_eq!(stats.cached_core_entries, 0);
}

#[test]
fn failed_semantic_and_interface_queries_do_not_insert_results() {
    let database = CompilerDatabase::default();
    assert!(compile(&database, function("broken", "missing")).is_err());
    assert!(compile(&database, function("broken", "missing")).is_err());
    assert!(database
        .module_interface_shared(
            LoadedSource {
                id: "entry.veac".into(),
                source: ENTRY.into(),
            },
            &MemoryLoader::default(),
        )
        .is_err());
    assert!(database
        .module_interface_shared(
            LoadedSource {
                id: "missing-import.veac".into(),
                source: "module { import \"./missing.veac\" as missing; }".into(),
            },
            &MemoryLoader::default(),
        )
        .is_err());

    let stats = database.statistics();
    assert_eq!(stats.hir_insertions, 0);
    assert_eq!(stats.core_insertions, 0);
    assert_eq!(stats.interface_insertions, 0);
}

#[test]
fn clear_releases_all_four_cache_layers() {
    let database = CompilerDatabase::default();
    database.parse("entry.veac", ENTRY).unwrap();
    compile(&database, function("one", "1")).unwrap();
    database
        .module_interface_shared(module(), &MemoryLoader::default())
        .unwrap();
    let before = database.statistics();
    assert!(before.cached_syntax_entries > 0);
    assert!(before.cached_interface_entries > 0);
    assert!(before.cached_hir_entries > 0);
    assert!(before.cached_core_entries > 0);

    database.clear();
    let stats = database.statistics();
    assert_eq!(stats.cached_syntax_entries, 0);
    assert_eq!(stats.cached_syntax_source_bytes, 0);
    assert_eq!(stats.cached_interface_entries, 0);
    assert_eq!(stats.cached_interface_retained_bytes, 0);
    assert_eq!(stats.cached_hir_entries, 0);
    assert_eq!(stats.cached_hir_retained_bytes, 0);
    assert_eq!(stats.cached_core_entries, 0);
    assert_eq!(stats.cached_core_retained_bytes, 0);
}

fn syntax_bytes(path: &str, source: &str) -> usize {
    let database = CompilerDatabase::default();
    database.parse(path, source).unwrap();
    database.statistics().cached_syntax_source_bytes
}

fn module() -> LoadedSource {
    LoadedSource {
        id: "module.veac".into(),
        source: MODULE.into(),
    }
}

fn function(name: &str, body: &str) -> FunctionDefinition {
    FunctionDefinition::new(
        name,
        Vec::new(),
        ValueType::primitive(PrimitiveType::Integer),
        format!("{{ {body} }}"),
    )
}

fn compile(
    database: &CompilerDatabase,
    definition: FunctionDefinition,
) -> Result<ExpressionContext, crate::program::expression::ExpressionError> {
    database.compile_functions(&ExpressionContext::empty(), &[definition], usize::MAX)
}
