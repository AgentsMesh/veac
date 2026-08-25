use std::sync::{mpsc, Arc};
use std::time::Duration;

use super::interface_key::InterfaceQueryKey;
use super::key::SyntaxQueryKey;
use super::*;
use crate::program::expression::{
    CompiledFunctionBatch, ExpressionContext, FunctionDefinition, FunctionQueryKey, PrimitiveType,
    TypedFunctionBatch, ValueType,
};
use crate::program::loader::{LoadedSource, MemoryLoader};
use crate::program::model::SurfaceFile;
use crate::program::ModuleInterface;

const MODULE: &str = "module { export fn identity(value: int) -> int { value } }";
const SOURCE: &str =
    "fn main(context: Context) -> Project { project(identifier(\"p\"), project_settings(1)) }";

struct CachedValues {
    epoch: lifecycle::QueryEpoch,
    syntax_key: SyntaxQueryKey,
    syntax: Arc<SurfaceFile>,
    interface_key: InterfaceQueryKey,
    interface: Arc<ModuleInterface>,
    function_key: FunctionQueryKey,
    hir: Arc<TypedFunctionBatch>,
    core: Arc<CompiledFunctionBatch>,
}

#[test]
fn clear_waits_for_an_active_cache_admission() {
    let database = Arc::new(CompilerDatabase::default());
    let admission = database.lifecycle.read().unwrap();
    let (started_tx, started_rx) = mpsc::channel();
    let (cleared_tx, cleared_rx) = mpsc::channel();
    let worker = Arc::clone(&database);
    let thread = std::thread::spawn(move || {
        started_tx.send(()).unwrap();
        worker.clear();
        cleared_tx.send(()).unwrap();
    });

    started_rx.recv().unwrap();
    assert!(cleared_rx.recv_timeout(Duration::from_millis(50)).is_err());
    drop(admission);
    cleared_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    thread.join().unwrap();
}

#[test]
fn clear_rejects_late_insertions_at_every_query_layer() {
    let database = CompilerDatabase::default();
    let cached = populate(&database);
    database.clear();
    let before = database.statistics();

    reinsert(&database, cached);
    assert_empty_with_bypasses(&database, before);
}

#[test]
fn dependency_reset_rejects_every_old_epoch_insertion() {
    let database = CompilerDatabase::with_limits(CompilerDatabaseLimits {
        dependency_entries: 2,
        dependency_retained_bytes: usize::MAX,
        ..Default::default()
    });
    let cached = populate(&database);
    database.parse("reset.veac", SOURCE).unwrap();
    assert_eq!(database.statistics().dependency_resets, 1);
    let before = database.statistics();

    reinsert(&database, cached);
    let stats = database.statistics();
    assert_eq!(stats.cached_interface_entries, 0);
    assert_eq!(stats.cached_hir_entries, 0);
    assert_eq!(stats.cached_core_entries, 0);
    assert_bypasses(stats, before);
}

fn populate(database: &CompilerDatabase) -> CachedValues {
    let module = LoadedSource {
        id: "module.veac".into(),
        source: MODULE.into(),
    };
    let interface = database
        .module_interface_shared(module, &MemoryLoader::default())
        .unwrap();
    let interface_key = InterfaceQueryKey::new(
        "module.veac",
        [(
            "module.veac".into(),
            database.source_revision("module.veac", MODULE),
        )],
        [],
    );
    let definitions = [function()];
    let context = ExpressionContext::empty();
    database
        .compile_functions(&context, &definitions, usize::MAX)
        .unwrap();
    let function_key = FunctionQueryKey::new(&context, &definitions);
    let epoch = database.query_epoch();
    let hir = database.cached_hir(&epoch, &function_key).unwrap();
    let core = database.cached_core(&epoch, &function_key).unwrap();
    let syntax = database.parse("late.veac", SOURCE).unwrap();
    CachedValues {
        epoch: database.query_epoch(),
        syntax_key: SyntaxQueryKey::new("late.veac", SOURCE),
        syntax,
        interface_key,
        interface,
        function_key,
        hir,
        core,
    }
}

fn reinsert(database: &CompilerDatabase, cached: CachedValues) {
    database.cache_syntax(&cached.epoch, cached.syntax_key, 1, cached.syntax);
    database.cache_interface(
        &cached.epoch,
        cached.interface_key,
        Some(1),
        cached.interface,
    );
    database.cache_hir(
        &cached.epoch,
        cached.function_key.clone(),
        Some(1),
        cached.hir,
    );
    database.cache_core(&cached.epoch, cached.function_key, Some(1), cached.core);
}

fn assert_empty_with_bypasses(database: &CompilerDatabase, before: CompilerDatabaseStatistics) {
    let stats = database.statistics();
    assert_eq!(stats.cached_syntax_entries, 0);
    assert_eq!(stats.cached_interface_entries, 0);
    assert_eq!(stats.cached_hir_entries, 0);
    assert_eq!(stats.cached_core_entries, 0);
    assert_eq!(stats.dependency_retained_entries, 0);
    assert_eq!(stats.dependency_retained_bytes, 0);
    assert_bypasses(stats, before);
}

fn assert_bypasses(after: CompilerDatabaseStatistics, before: CompilerDatabaseStatistics) {
    assert_eq!(after.syntax_bypasses, before.syntax_bypasses + 1);
    assert_eq!(after.interface_bypasses, before.interface_bypasses + 1);
    assert_eq!(after.hir_bypasses, before.hir_bypasses + 1);
    assert_eq!(after.core_bypasses, before.core_bypasses + 1);
}

fn function() -> FunctionDefinition {
    FunctionDefinition::new(
        "one",
        Vec::new(),
        ValueType::primitive(PrimitiveType::Integer),
        "{ 1 }",
    )
}
