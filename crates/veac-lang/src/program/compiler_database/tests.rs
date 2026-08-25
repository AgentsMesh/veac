use std::sync::Arc;

use super::*;

const SOURCE: &str =
    "fn main(context: Context) -> Project { project(identifier(\"p\"), project_settings(1)) }";

#[test]
fn repeated_syntax_queries_reuse_the_verified_surface() {
    let database = CompilerDatabase::default();
    let first = database.parse("main.veac", SOURCE).unwrap();
    let second = database.parse("main.veac", SOURCE).unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(database.statistics().syntax_hits, 1);
    assert_eq!(database.statistics().syntax_misses, 1);
}

#[test]
fn changed_source_and_source_identity_have_distinct_queries() {
    let database = CompilerDatabase::default();
    database.parse("main.veac", SOURCE).unwrap();
    database
        .parse("main.veac", &SOURCE.replace("settings(1)", "settings(2)"))
        .unwrap();
    database.parse("other.veac", SOURCE).unwrap();
    let stats = database.statistics();
    assert_eq!(stats.syntax_insertions, 3);
    assert_eq!(stats.cached_syntax_entries, 3);
}

#[test]
fn bounded_cache_evicts_old_entries_and_can_be_cleared() {
    let database = CompilerDatabase::with_limits(CompilerDatabaseLimits {
        syntax_entries: 1,
        syntax_source_bytes: usize::MAX,
        ..Default::default()
    });
    database.parse("first.veac", SOURCE).unwrap();
    database.parse("second.veac", SOURCE).unwrap();
    assert_eq!(database.statistics().syntax_evictions, 1);
    database.clear();
    let stats = database.statistics();
    assert_eq!(stats.cached_syntax_entries, 0);
    assert_eq!(stats.syntax_insertions, 2);
    assert_eq!(stats.pending_semantic_invalidations, 0);
    assert_eq!(stats.semantic_invalidations, 0);
}

#[test]
fn over_limit_queries_bypass_cache_without_failing_parse() {
    let database = CompilerDatabase::with_limits(CompilerDatabaseLimits {
        syntax_entries: 0,
        syntax_source_bytes: 0,
        ..Default::default()
    });
    database.parse("main.veac", SOURCE).unwrap();
    database.parse("main.veac", SOURCE).unwrap();
    let stats = database.statistics();
    assert_eq!(stats.syntax_bypasses, 2);
    assert_eq!(stats.syntax_misses, 2);
}

#[test]
fn invalid_syntax_is_never_cached() {
    let database = CompilerDatabase::default();
    assert!(database.parse("main.veac", "fn").is_err());
    assert!(database.parse("main.veac", "fn").is_err());
    assert_eq!(database.statistics().syntax_misses, 2);
    assert_eq!(database.statistics().syntax_insertions, 0);
}

#[test]
fn concurrent_identical_queries_retain_one_surface() {
    let database = Arc::new(CompilerDatabase::default());
    let threads = (0..4)
        .map(|_| {
            let database = Arc::clone(&database);
            std::thread::spawn(move || database.parse("main.veac", SOURCE).unwrap())
        })
        .collect::<Vec<_>>();
    let parsed = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect::<Vec<_>>();
    assert!(parsed
        .windows(2)
        .all(|pair| Arc::ptr_eq(&pair[0], &pair[1])));
    assert_eq!(database.statistics().cached_syntax_entries, 1);
    assert_eq!(database.statistics().syntax_insertions, 1);
}

#[test]
fn source_revisions_are_stable_and_bind_source_identity() {
    let database = CompilerDatabase::default();
    let first = database.source_revision("main.veac", SOURCE);
    assert_eq!(first, database.source_revision("main.veac", SOURCE));
    assert_ne!(first, database.source_revision("other.veac", SOURCE));
    assert_ne!(
        first,
        database.source_revision("main.veac", &format!("{SOURCE}\n"))
    );
    assert_eq!(first.sha256().len(), 64);
}
