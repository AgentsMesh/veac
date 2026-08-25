use super::*;
use crate::program::loader::{LoadedSource, MemoryLoader};

const SOURCE: &str = r#"
fn main(context: Context) -> Project {
  let timeline = sequence(
    identifier("main"), "数据库 facade",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
  );
  project(identifier("database"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#;

#[test]
fn database_build_facade_executes_source_and_loader_entries() {
    let database = CompilerDatabase::default();
    database.build_source(SOURCE).unwrap();
    database
        .build_with_loader(
            LoadedSource {
                id: "main.veac".into(),
                source: SOURCE.into(),
            },
            &MemoryLoader::default(),
        )
        .unwrap();
    assert!(database.statistics().syntax_hits >= 1);
}

#[test]
fn cached_preparation_still_executes_in_fresh_graph_transactions() {
    let database = CompilerDatabase::default();
    let first = database.prepare_source(SOURCE).unwrap().execute().unwrap();
    let second = database.prepare_source(SOURCE).unwrap().execute().unwrap();
    assert!(!std::ptr::eq(first.graph().root(), second.graph().root()));
    assert_eq!(
        veac_ir::canonical_json(first.envelope()).unwrap(),
        veac_ir::canonical_json(second.envelope()).unwrap()
    );
}
