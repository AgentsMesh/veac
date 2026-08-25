use std::fs;

use tempfile::tempdir;
use veac_lang::program::{BuildInputManifestV1, CompilerDatabase};

const SOURCE: &str = r#"
fn main(context: Context) -> Project {
  let timeline = sequence(
    identifier("main"), "数据库公开入口",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
  );
  project(identifier("database-facade"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#;

#[test]
fn database_path_facades_preserve_root_and_input_execution() {
    let directory = tempdir().unwrap();
    let entry = directory.path().join("main.veac");
    fs::write(&entry, SOURCE).unwrap();
    let database = CompilerDatabase::default();

    let (root, prepared) = database.prepare_path_with_root(&entry).unwrap();
    assert_eq!(root, directory.path().canonicalize().unwrap());
    prepared.execute().unwrap();

    database
        .build_path_with_inputs(&entry, &BuildInputManifestV1::empty())
        .unwrap();
    let (built_root, built) = database.build_path_with_root(&entry).unwrap();
    assert_eq!(built_root, root);
    assert_eq!(built.root_module(), "main.veac");
    assert_eq!(database.statistics().syntax_hits, 2);
}

#[test]
fn database_source_input_facade_uses_the_same_query_contract() {
    let database = CompilerDatabase::default();
    database
        .build_source_with_inputs(SOURCE, &BuildInputManifestV1::empty())
        .unwrap();
    database.build_source(SOURCE).unwrap();
    assert_eq!(database.statistics().syntax_hits, 1);
}
