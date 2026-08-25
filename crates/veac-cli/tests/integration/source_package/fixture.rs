use std::path::{Path, PathBuf};

use super::*;
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef, SourceRevision,
};

pub(super) const PACKAGE_ROOT: &str = "veac-components@0.1.0";

pub(super) fn standard_package() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib/veac-components")
}

pub(super) fn project(temp: &TempDir) -> PathBuf {
    let source = format!(
        "import \"package:veac-components@0.1.0/main.veac\" as components;\n\
         fn duration() -> time {{ 200ms }}\n{}",
        GENERATED_SOURCE.replace("during(0s, 200ms)", "during(0s, duration())")
    );
    source_file(temp, &source)
}

pub(super) fn revision(source: &Path, package: &Path) -> SourceRevision {
    let output = veac()
        .args([
            "source-revision",
            source.to_str().unwrap(),
            "--package-root",
        ])
        .arg(package)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

pub(super) fn edit_batch(source: &Path, package: &Path) -> SourceEditBatch {
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_package_cli_edit").unwrap(),
        revision(source, package),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", "duration"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ 400ms }".into(),
        },
    });
    batch
}

pub(super) fn copy_package(source: &Path, destination: &Path) {
    for relative in [
        "main.veac",
        "layout.veac",
        "text.veac",
        "motion.veac",
        "media.veac",
        "audio.veac",
        "delivery.veac",
        "components/card.veac",
        "veac.package.api.json",
        "veac.package.json",
        "veac.package.lock",
    ] {
        let target = destination.join(relative);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(source.join(relative), target).unwrap();
    }
}
