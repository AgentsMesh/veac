use super::{capture, check_root_module};
use crate::ProjectPackageSet;
use veac_project::ProjectPath;

const ENTRY: &str = r#"import "./title.veac" as title;
fn main(context: Context) -> Project {
  let timeline = sequence(identifier("main"), title.value(),
    sequence_settings(canvas(16px, 16px), frame_rate(1, 1), 8000));
  project(identifier("test"), project_settings(60))
    .with_sequence(timeline).entry(timeline)
}
"#;

#[test]
fn source_graph_root_identity_is_exact() {
    check_root_module("main.veac", "main.veac").unwrap();
    let error = check_root_module("other.veac", "main.veac").unwrap_err();
    assert!(error.message().contains("unexpected root module"));
}

#[test]
fn nested_entry_keeps_entry_local_module_identity() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    std::fs::create_dir_all(source.join("nested")).unwrap();
    std::fs::write(source.join("nested/main.veac"), ENTRY).unwrap();
    std::fs::write(
        source.join("nested/title.veac"),
        "module { export fn value() -> text { \"Nested\" } }\n",
    )
    .unwrap();

    let (_, revision) = capture(
        &source,
        &ProjectPath::new("nested/main.veac"),
        &ProjectPackageSet::capture(&[]).unwrap(),
    )
    .unwrap();
    assert_eq!(revision.root_module, "main.veac");
    assert_eq!(revision.authored_modules, ["main.veac", "title.veac"]);
}
