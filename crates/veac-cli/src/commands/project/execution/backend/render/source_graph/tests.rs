use super::{prepare, verify, verify_prepared};
use veac_build::{ProjectFileSnapshot, ProjectSourceGraphRevision};

const ENTRY: &str = r#"import "./title.veac" as title;
fn main(context: Context) -> Project {
  let timeline = sequence(identifier("main"), title.value(),
    sequence_settings(canvas(16px, 16px), frame_rate(1, 1), 8000));
  project(identifier("test"), project_settings(60))
    .with_sequence(timeline).entry(timeline)
}
"#;
const MODULE: &str = "module { export fn value() -> text { \"One\" } }\n";

fn fixture() -> (
    tempfile::TempDir,
    ProjectFileSnapshot,
    ProjectSourceGraphRevision,
) {
    fixture_at("")
}

fn fixture_at(
    directory: &str,
) -> (
    tempfile::TempDir,
    ProjectFileSnapshot,
    ProjectSourceGraphRevision,
) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join(directory);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("main.veac"), ENTRY).unwrap();
    std::fs::write(root.join("title.veac"), MODULE).unwrap();
    let prepared = veac_lang::program::prepare_path(&root.join("main.veac")).unwrap();
    let authored_sources = prepared.source_graph().project_sources();
    let modules = authored_sources
        .iter()
        .map(|(path, source)| veac_lang::source_edit::SourceModule::utf8(path, source))
        .collect::<Vec<_>>();
    let revision = veac_lang::source_edit::source_graph_revision(&modules).unwrap();
    (
        temp,
        ProjectFileSnapshot {
            path: if directory.is_empty() {
                "main.veac".to_owned()
            } else {
                format!("{directory}/main.veac")
            },
            content: veac_artifact::ContentDigest::sha256(ENTRY.as_bytes()),
            size_bytes: ENTRY.len() as u64,
        },
        ProjectSourceGraphRevision {
            root_module: "main.veac".to_owned(),
            authored_source_graph_sha256: revision.source_graph_sha256,
            complete_source_graph_sha256: prepared
                .source_graph()
                .complete_revision()
                .sha256()
                .to_owned(),
            authored_module_count: 2,
            authored_modules: vec!["main.veac".to_owned(), "title.veac".to_owned()],
        },
    )
}

fn packages() -> veac_build::ProjectPackageSet {
    veac_build::ProjectPackageSet::capture(&[]).unwrap()
}

#[test]
fn prepared_graph_must_match_the_closed_revision() {
    let (temp, entry, revision) = fixture();
    let packages = packages();
    let prepared = prepare(temp.path(), &packages, &entry, &revision).unwrap();
    verify_prepared(&prepared, &entry, &revision).unwrap();
    verify(temp.path(), &packages, &entry, &revision).unwrap();

    let mut changed = revision.clone();
    changed.root_module = "other.veac".to_owned();
    assert!(verify_prepared(&prepared, &entry, &changed)
        .unwrap_err()
        .message()
        .contains("root"));
    changed = revision.clone();
    changed.authored_module_count = 1;
    assert!(verify_prepared(&prepared, &entry, &changed)
        .unwrap_err()
        .message()
        .contains("inventory"));
    changed = revision.clone();
    changed.authored_modules.pop();
    assert!(verify_prepared(&prepared, &entry, &changed)
        .unwrap_err()
        .message()
        .contains("inventory"));
    changed = revision.clone();
    changed.authored_source_graph_sha256 = "0".repeat(64);
    assert!(verify_prepared(&prepared, &entry, &changed)
        .unwrap_err()
        .message()
        .contains("revision"));
    changed = revision.clone();
    changed.complete_source_graph_sha256 = "0".repeat(64);
    assert!(verify_prepared(&prepared, &entry, &changed)
        .unwrap_err()
        .message()
        .contains("complete"));
}

#[test]
fn imported_module_changes_are_detected_before_execution() {
    let (temp, entry, revision) = fixture();
    std::fs::write(
        temp.path().join("title.veac"),
        "module { export fn value() -> text { \"Two\" } }\n",
    )
    .unwrap();
    let error = prepare(temp.path(), &packages(), &entry, &revision).unwrap_err();
    assert!(error.message().contains("revision"));
}

#[test]
fn nested_target_keeps_entry_local_module_identity() {
    let (temp, entry, revision) = fixture_at("nested");
    let prepared = prepare(temp.path(), &packages(), &entry, &revision).unwrap();
    assert_eq!(prepared.root_module(), "main.veac");
    assert_eq!(
        prepared
            .source_graph()
            .project_sources()
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        ["main.veac", "title.veac"]
    );
}

#[test]
fn entry_snapshot_and_language_errors_fail_closed() {
    let (temp, mut entry, revision) = fixture();
    entry.content = veac_artifact::ContentDigest::sha256(b"wrong");
    assert!(prepare(temp.path(), &packages(), &entry, &revision).is_err());

    let broken = "fn main(context: Context) -> Project { missing }\n";
    std::fs::write(temp.path().join("main.veac"), broken).unwrap();
    entry.content = veac_artifact::ContentDigest::sha256(broken.as_bytes());
    entry.size_bytes = broken.len() as u64;
    assert!(prepare(temp.path(), &packages(), &entry, &revision)
        .unwrap_err()
        .message()
        .contains("preparation"));
}
