use super::FunctionOrigin;
use crate::program::{CompilerDatabase, CompilerDatabaseLimits};
use std::sync::Arc;

const SOURCE: &str =
    "fn main(context: Context) -> Project { project(identifier(\"p\"), project_settings(1)) }";

fn origin_from(
    database: &CompilerDatabase,
) -> (FunctionOrigin, Arc<crate::program::model::SurfaceFile>) {
    let file = database.parse("main.veac", SOURCE).unwrap();
    let body = &file.functions[0].body;
    (
        FunctionOrigin::new("main.veac", body.span.start..body.span.end)
            .with_syntax(&file.syntax, &body.syntax),
        file,
    )
}

#[test]
fn syntax_provenance_survives_syntax_cache_eviction() {
    let database = CompilerDatabase::with_limits(CompilerDatabaseLimits {
        syntax_entries: 1,
        syntax_source_bytes: usize::MAX,
        ..Default::default()
    });
    let (origin, file) = origin_from(&database);
    database.parse("other.veac", SOURCE).unwrap();
    assert_eq!(database.statistics().syntax_evictions, 1);
    drop(file);

    assert!(origin.syntax().expect("authored syntax").parse().is_ok());
}

#[test]
fn syntax_provenance_survives_database_clear() {
    let database = CompilerDatabase::default();
    let (origin, file) = origin_from(&database);
    database.clear();
    assert_eq!(database.statistics().cached_syntax_entries, 0);
    drop(file);

    assert!(origin.syntax().expect("authored syntax").parse().is_ok());
}

#[test]
fn detached_provenance_keeps_location_without_retaining_syntax() {
    let database = CompilerDatabase::default();
    let (origin, _) = origin_from(&database);
    let detached = origin.detached();
    assert_eq!(detached.source_id(), origin.source_id());
    assert_eq!(detached.body_span(), origin.body_span());
    assert!(detached.syntax().is_none());
}
