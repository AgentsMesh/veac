use super::super::support::*;
use super::package_contract_fixture;
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef, SourceRevision,
};

#[test]
fn every_source_frontend_rejects_exact_dependency_split_brain_in_any_root_order() {
    let project = tempdir().unwrap();
    let packages = tempdir().unwrap();
    let source = source_file(
        &project,
        &format!(
            "import \"package:owner@1.0.0/main.veac\" as owner;\n\
             import \"package:shared@1.0.0/main.veac\" as shared;\n{GENERATED_SOURCE}"
        ),
    );
    let owner = packages.path().join("owner");
    let shared = packages.path().join("shared");
    package_contract_fixture::write_split_brain_pair(&owner, &shared);

    for command in ["check", "build", "fmt", "source-revision", "source-index"] {
        for roots in [
            [owner.as_path(), shared.as_path()],
            [shared.as_path(), owner.as_path()],
        ] {
            veac()
                .arg(command)
                .arg(&source)
                .arg("--package-root")
                .arg(roots[0])
                .arg("--package-root")
                .arg(roots[1])
                .assert()
                .failure()
                .stderr(predicate::str::contains("LANG_PACKAGE_CONTRACT"))
                .stderr(predicate::str::contains(
                    "conflicting content or API contracts",
                ));
        }
    }

    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_split_brain").unwrap(),
        SourceRevision {
            authored_source_graph_sha256: "0".repeat(64),
            complete_source_graph_sha256: "0".repeat(64),
        },
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", "main"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ 0 }".into(),
        },
    });
    let batch_path = project.path().join("edit.json");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&batch).unwrap(),
    )
    .unwrap();
    for roots in [
        [owner.as_path(), shared.as_path()],
        [shared.as_path(), owner.as_path()],
    ] {
        veac()
            .arg("source-edit")
            .arg(&source)
            .arg(&batch_path)
            .arg("--package-root")
            .arg(roots[0])
            .arg("--package-root")
            .arg(roots[1])
            .assert()
            .failure()
            .stderr(predicate::str::contains("LANG_PACKAGE_CONTRACT"));
    }
}
