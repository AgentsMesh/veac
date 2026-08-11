use super::{destination_path, render_source_graph_inputs, root_source_graph_inputs};

#[test]
fn receipt_destination_stays_in_build_authority_and_protects_inputs() {
    let temp = tempfile::tempdir().unwrap();
    let build = temp.path().join("build");
    std::fs::create_dir(&build).unwrap();
    let build = std::fs::canonicalize(build).unwrap();
    let protected = temp.path().join("project.veac");
    std::fs::write(&protected, b"source").unwrap();

    let receipt = build.join("receipt.json");
    assert_eq!(
        destination_path(&receipt, &build, std::slice::from_ref(&protected)).unwrap(),
        receipt
    );
    let outside = temp.path().join("receipt.json");
    let error = destination_path(&outside, &build, std::slice::from_ref(&protected)).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "PROJECT_RECEIPT_PATH");

    let error = destination_path(&protected, &build, &[protected.clone()]).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "OUTPUT_OVERWRITES_INPUT");
}

#[cfg(unix)]
#[test]
fn receipt_destination_rejects_a_hard_link_to_an_input() {
    let temp = tempfile::tempdir().unwrap();
    let build = temp.path().join("build");
    let source = temp.path().join("source/nested");
    std::fs::create_dir(&build).unwrap();
    std::fs::create_dir_all(&source).unwrap();
    let protected = source.join("helper.veac");
    let alias = build.join("receipt.json");
    std::fs::write(&protected, b"source").unwrap();
    std::fs::hard_link(&protected, &alias).unwrap();
    let revision = veac_build::ProjectSourceGraphRevision {
        root_module: "main.veac".to_owned(),
        source_graph_sha256: "0".repeat(64),
        module_count: 2,
        modules: vec!["helper.veac".to_owned(), "main.veac".to_owned()],
    };
    let protected =
        render_source_graph_inputs(&temp.path().join("source"), "nested/main.veac", &revision);

    let error = destination_path(&alias, &build, &protected).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "OUTPUT_OVERWRITES_INPUT");
}

#[cfg(unix)]
#[test]
fn nested_evidence_modules_are_protected_from_receipt_aliases() {
    let temp = tempfile::tempdir().unwrap();
    let build = temp.path().join("build");
    let source = temp.path().join("source");
    std::fs::create_dir(&build).unwrap();
    std::fs::create_dir_all(source.join("contracts")).unwrap();
    let helper = source.join("contracts/helper.veac");
    let alias = build.join("receipt.json");
    std::fs::write(&helper, b"source").unwrap();
    std::fs::hard_link(&helper, &alias).unwrap();
    let revision = veac_build::ProjectSourceGraphRevision {
        root_module: "contracts/evidence.veac".to_owned(),
        source_graph_sha256: "0".repeat(64),
        module_count: 2,
        modules: vec![
            "contracts/evidence.veac".to_owned(),
            "contracts/helper.veac".to_owned(),
        ],
    };
    let protected = root_source_graph_inputs(&source, &revision);

    let error = destination_path(&alias, &build, &protected).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "OUTPUT_OVERWRITES_INPUT");
}
