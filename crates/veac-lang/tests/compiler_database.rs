use std::collections::BTreeMap;

use veac_lang::program::{CompilerDatabase, CompilerDatabaseLimits, LoadedSource, SourceLoader};

#[path = "program_functions/support.rs"]
mod support;

#[derive(Default)]
struct Loader {
    sources: BTreeMap<String, String>,
}

impl SourceLoader for Loader {
    fn load(&self, _importer: &str, requested: &str) -> Result<LoadedSource, String> {
        let id = requested.trim_start_matches("./").to_owned();
        self.sources
            .get(&id)
            .cloned()
            .map(|source| LoadedSource { id, source })
            .ok_or_else(|| format!("missing module `{requested}`"))
    }

    fn authority(&self, _source_id: &str) -> veac_lang::program::SourceAuthority {
        veac_lang::program::SourceAuthority::Project
    }
}

fn entry(declarations: &str, duration: &str) -> LoadedSource {
    LoadedSource {
        id: "main.veac".to_owned(),
        source: support::project_with(declarations, duration),
    }
}

fn timing_module(duration: &str) -> String {
    format!("module {{ export fn duration() -> time {{ {duration} }} }}")
}

#[test]
fn cached_and_clean_builds_publish_identical_canonical_programs() {
    let source = support::project_with("", "1250ms");
    let database = CompilerDatabase::default();
    let first = database.prepare_source(&source).unwrap();
    let second = database.prepare_source(&source).unwrap();
    let clean = CompilerDatabase::default().prepare_source(&source).unwrap();
    assert_eq!(
        first.source_index().unwrap().revision(),
        second.source_index().unwrap().revision()
    );
    assert_eq!(
        first.source_index().unwrap().revision(),
        clean.source_index().unwrap().revision()
    );
    let first = first.execute().unwrap();
    let second = second.execute().unwrap();
    let clean = clean.execute().unwrap();
    let canonical = veac_ir::canonical_json(first.envelope()).unwrap();
    assert_eq!(
        canonical,
        veac_ir::canonical_json(second.envelope()).unwrap()
    );
    assert_eq!(
        canonical,
        veac_ir::canonical_json(clean.envelope()).unwrap()
    );
    assert_eq!(database.statistics().syntax_hits, 1);
}

#[test]
fn imported_source_graphs_reuse_each_exact_syntax_query() {
    let loader = Loader {
        sources: BTreeMap::from([("timing.veac".into(), timing_module("1s"))]),
    };
    let root = entry("import \"./timing.veac\" as timing;", "timing.duration()");
    let database = CompilerDatabase::default();
    database.prepare_with_loader(root.clone(), &loader).unwrap();
    database.prepare_with_loader(root, &loader).unwrap();
    let stats = database.statistics();
    assert_eq!((stats.syntax_misses, stats.syntax_hits), (2, 2));
    assert_eq!(stats.cached_syntax_entries, 2);
    assert_eq!(
        database
            .dependency_snapshot("main.veac")
            .unwrap()
            .direct_dependencies,
        ["timing.veac"]
    );
}

#[test]
fn source_content_and_identity_invalidate_syntax_queries() {
    let database = CompilerDatabase::default();
    let root = entry("import \"./timing.veac\" as timing;", "timing.duration()");
    let one = Loader {
        sources: BTreeMap::from([("timing.veac".into(), timing_module("1s"))]),
    };
    let two = Loader {
        sources: BTreeMap::from([("timing.veac".into(), timing_module("2s"))]),
    };
    let first = database
        .prepare_with_loader(root.clone(), &one)
        .unwrap()
        .execute()
        .unwrap();
    let second = database
        .prepare_with_loader(root, &two)
        .unwrap()
        .execute()
        .unwrap();
    assert_eq!(support::result_duration(&first), "1s");
    assert_eq!(support::result_duration(&second), "2s");

    let source = support::project_with("", "1s");
    for id in ["one.veac", "two.veac"] {
        database
            .prepare_with_loader(
                LoadedSource {
                    id: id.into(),
                    source: source.clone(),
                },
                &Loader::default(),
            )
            .unwrap();
    }
    assert_eq!(database.statistics().cached_syntax_entries, 5);
}

#[test]
fn dependency_change_invalidates_importers_but_preserves_clean_equivalence() {
    let database = CompilerDatabase::default();
    let root = entry("import \"./timing.veac\" as timing;", "timing.duration()");
    let one = Loader {
        sources: BTreeMap::from([("timing.veac".into(), timing_module("1s"))]),
    };
    let two = Loader {
        sources: BTreeMap::from([("timing.veac".into(), timing_module("2s"))]),
    };
    database.prepare_with_loader(root.clone(), &one).unwrap();
    let cached = database
        .prepare_with_loader(root.clone(), &two)
        .unwrap()
        .execute()
        .unwrap();
    let clean = CompilerDatabase::default()
        .prepare_with_loader(root, &two)
        .unwrap()
        .execute()
        .unwrap();
    assert_eq!(
        veac_ir::canonical_json(cached.envelope()).unwrap(),
        veac_ir::canonical_json(clean.envelope()).unwrap()
    );
}

#[test]
fn semantic_failures_are_deterministic_but_invalid_syntax_is_not_cached() {
    let database = CompilerDatabase::default();
    let semantic = support::project_with("", "missing_duration()");
    let first = database.prepare_source(&semantic).unwrap_err();
    let second = database.prepare_source(&semantic).unwrap_err();
    assert_eq!(first, second);
    assert_eq!(database.statistics().syntax_hits, 1);

    let first = database.prepare_source("fn").unwrap_err();
    let second = database.prepare_source("fn").unwrap_err();
    assert_eq!(first, second);
    assert_eq!(database.statistics().syntax_misses, 3);
}

#[test]
fn cache_capacity_never_changes_compilation_semantics() {
    let database = CompilerDatabase::with_limits(CompilerDatabaseLimits {
        syntax_entries: 0,
        syntax_source_bytes: 0,
        ..Default::default()
    });
    let source = support::project_with("", "750ms");
    let first = database.prepare_source(&source).unwrap().execute().unwrap();
    let second = database.prepare_source(&source).unwrap().execute().unwrap();
    assert_eq!(
        support::result_duration(&first),
        support::result_duration(&second)
    );
    assert_eq!(database.statistics().syntax_bypasses, 2);
}

#[path = "compiler_database/semantic_queries.rs"]
mod semantic_queries;
