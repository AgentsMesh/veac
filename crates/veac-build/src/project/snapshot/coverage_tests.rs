use std::collections::BTreeMap;

use super::{fingerprint, graph_module_count, graph_root_source, io_error, ProjectRoots};
use veac_project::ProjectPath;

#[test]
fn malformed_source_graphs_keep_their_action_context() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let material = temp.path().join("material");
    std::fs::create_dir(&source).unwrap();
    std::fs::create_dir(&material).unwrap();
    std::fs::write(source.join("main.veac"), "{").unwrap();
    std::fs::write(source.join("evidence.veac"), "{").unwrap();
    let roots = ProjectRoots::new(&source, &material).unwrap();

    let render = roots
        .source_graph(&ProjectPath::new("main.veac"))
        .unwrap_err();
    assert!(render.message().contains("VEAC source graph"));
    let evidence = roots
        .evidence_graph(&ProjectPath::new("evidence.veac"))
        .unwrap_err();
    assert!(evidence.message().contains("evidence source graph"));
}

#[test]
fn snapshot_helper_failures_preserve_owned_context() {
    let error = io_error(String::from("owned"), std::io::Error::other("disk"));
    assert!(error.message().contains("owned"));
    let temp = tempfile::tempdir().unwrap();
    assert!(fingerprint(temp.path()).is_err());

    let sources = BTreeMap::from([("main.veac".to_owned(), "source".to_owned())]);
    assert_eq!(
        graph_root_source(&sources, "main.veac", "test").unwrap(),
        "source"
    );
    assert!(graph_root_source(&sources, "missing.veac", "test").is_err());
    assert_eq!(graph_module_count(1, "test").unwrap(), 1);
    assert!(graph_module_count(usize::MAX, "test").is_err());
}
