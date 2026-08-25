use super::super::support::*;
use super::fixture::*;
use super::package_contract_fixture;
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef, SourceRevision,
};

#[test]
fn source_edit_rejects_a_package_module_as_read_only() {
    let temp = tempdir().unwrap();
    let source = project(&temp);
    let package = standard_package();
    let mut batch = edit_batch(&source, &package);
    if let SourceEditOperation::SetBody { target, .. } = &mut batch.operations[0] {
        target.module = format!("packages/{PACKAGE_ROOT}/main.veac");
        target.path = veac_lang::source_edit::SourceNodePath::Function {
            function: "card".into(),
        };
    }
    let batch_path = temp.path().join("package-read-only.json");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&batch).unwrap(),
    )
    .unwrap();
    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
            "--package-root",
        ])
        .arg(&package)
        .assert()
        .failure()
        .stderr(predicate::str::contains("SOURCE_EDIT_READ_ONLY_DEPENDENCY"));
}

#[test]
fn source_edit_output_cannot_write_inside_a_package_root() {
    let temp = tempdir().unwrap();
    let source = project(&temp);
    let package = standard_package();
    let batch_path = temp.path().join("package-output.json");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&edit_batch(&source, &package))
            .unwrap(),
    )
    .unwrap();
    let output = package.join("forbidden.veac");
    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
            "--output",
        ])
        .arg(&output)
        .args(["--package-root"])
        .arg(&package)
        .assert()
        .failure()
        .stderr(predicate::str::contains("OUTPUT_OVERWRITES_INPUT"));
    assert!(!output.exists());
}

#[test]
fn source_edit_cannot_reclassify_a_package_root_as_writable_project_source() {
    let temp = tempdir().unwrap();
    let package = temp.path().join("package");
    copy_package(&standard_package(), &package);
    let source = package.join("main.veac");
    let original = std::fs::read_to_string(&source).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_package_authority_overlap").unwrap(),
        SourceRevision {
            authored_source_graph_sha256: "0".repeat(64),
            complete_source_graph_sha256: "0".repeat(64),
        },
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", "card"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ CardConfig { key: key, at: at, duration: duration, title: title, detail: detail, tone: tone, } }".into(),
        },
    });
    let batch_path = temp.path().join("overlap-edit.json");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&batch).unwrap(),
    )
    .unwrap();

    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
            "--package-root",
        ])
        .arg(&package)
        .assert()
        .failure()
        .stderr(predicate::str::contains("LANG_PACKAGE_AUTHORITY"));
    assert_eq!(std::fs::read_to_string(source).unwrap(), original);
}

#[test]
fn duplicate_exact_package_identity_is_rejected() {
    let temp = tempdir().unwrap();
    let source = project(&temp);
    let package = standard_package();
    veac()
        .args(["check", source.to_str().unwrap(), "--package-root"])
        .arg(&package)
        .args(["--package-root"])
        .arg(&package)
        .assert()
        .failure()
        .stderr(predicate::str::contains("LANG_PACKAGE_CONTRACT"));
}

#[test]
fn distinct_explicit_package_roots_are_order_independent() {
    let project_temp = tempdir().unwrap();
    let package_temp = tempdir().unwrap();
    let source = project(&project_temp);
    let standard = standard_package();
    let auxiliary = package_temp.path().join("auxiliary-package");
    package_contract_fixture::write_exact_package(&auxiliary, "auxiliary", "1.0.0", false);
    let original = std::fs::read_to_string(&source).unwrap();
    std::fs::write(
        &source,
        format!("import \"package:auxiliary@1.0.0/main.veac\" as auxiliary;\n{original}"),
    )
    .unwrap();
    let mut outputs = Vec::new();
    for roots in [
        [standard.as_path(), auxiliary.as_path()],
        [auxiliary.as_path(), standard.as_path()],
    ] {
        let output = veac()
            .args(["source-index", source.to_str().unwrap(), "--package-root"])
            .arg(roots[0])
            .args(["--package-root"])
            .arg(roots[1])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        outputs.push(serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap());
    }
    assert_eq!(outputs[0], outputs[1]);
}

#[test]
fn tampered_package_source_fails_during_build_and_check() {
    let temp = tempdir().unwrap();
    let package = temp.path().join("tampered-package");
    copy_package(&standard_package(), &package);
    std::fs::write(package.join("main.veac"), "module {}\n").unwrap();
    let source = project(&temp);
    for command in ["build", "check"] {
        veac()
            .args([command, source.to_str().unwrap(), "--package-root"])
            .arg(&package)
            .assert()
            .failure()
            .stderr(predicate::str::contains("LANG_PACKAGE_DIGEST"));
    }
}
