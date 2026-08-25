use std::fs;
use std::path::{Path, PathBuf};

use veac_lang::package::api::{canonical_api_metadata_json, package_api_digest, ApiMetadataV1};
use veac_lang::package::{
    canonical_package_lock_json, canonical_package_manifest_json, sha256_bytes, ExactVersion,
    LockedFile, PackageIdentity, PackageLockV1, PackageManifestV1, PackageName,
    PackageSourceLoader,
};
use veac_lang::program::{
    prepare_executable_source_edit_with_loader, prepare_with_loader,
    reprepare_executable_source_edit_preview, BuildInputManifestV1, CompositeSourceLoader,
    FileSystemLoader, LoadedSource, SourceAuthority, SourceTransactionError,
};
use veac_lang::source_edit::{
    BodySite, BodySource, ImportSource, SourceEditBatch, SourceEditOperation, SourceModuleAnchor,
    SourceNodeRef,
};

#[path = "program_functions/support.rs"]
mod support;

const REQUEST: &str = "package:root@1.0.0/main.veac";
const SOURCE_ID: &str = "packages/root@1.0.0/main.veac";

#[test]
fn newly_imported_package_drift_changes_the_complete_candidate_graph() {
    let project = tempfile::tempdir().unwrap();
    let entry = write_project(&project, "fn duration() -> time { 1s }");
    let package_a = package("module {}\n");
    let (loader_a, root) = composite(&entry, package_a.path());
    let prepared = prepare_with_loader(root, &loader_a).unwrap();
    let mut batch = edit(&prepared, "op_new_package_graph");
    batch.operations.push(SourceEditOperation::InsertImport {
        module: "main.veac".into(),
        anchor: SourceModuleAnchor::ModuleStart,
        import: ImportSource {
            path: REQUEST.into(),
            alias: "package_root".into(),
        },
    });
    let preview = prepare_executable_source_edit_with_loader(&prepared, &batch, &loader_a)
        .unwrap()
        .execute(&BuildInputManifestV1::empty())
        .unwrap();
    assert!(preview.built.sources().contains_key(SOURCE_ID));
    assert!(!preview.candidate_sources().contains_key(SOURCE_ID));
    assert_eq!(
        preview.built.source_graph().authority(SOURCE_ID),
        SourceAuthority::ReadOnlyDependency
    );

    let package_b = package("module { }\n");
    let (loader_b, entry_b) = composite(&entry, package_b.path());
    let rebuilt = reprepare_executable_source_edit_preview(&preview, entry_b, &loader_b).unwrap();
    let revision = rebuilt.source_revision().unwrap();
    assert_eq!(
        revision.authored_source_graph_sha256,
        preview.new_revision.authored_source_graph_sha256
    );
    assert_ne!(
        revision.complete_source_graph_sha256,
        preview.new_revision.complete_source_graph_sha256
    );
    assert_ne!(rebuilt.source_graph(), preview.built.source_graph());
}

#[test]
fn same_package_identity_with_different_valid_bytes_is_stale() {
    let project = tempfile::tempdir().unwrap();
    let declarations =
        format!("import \"{REQUEST}\" as package_root;\nfn duration() -> time {{ 1s }}");
    let entry = write_project(&project, &declarations);
    let package_a = package("module {}\n");
    let (loader_a, root) = composite(&entry, package_a.path());
    let prepared = prepare_with_loader(root, &loader_a).unwrap();
    let mut batch = edit(&prepared, "op_existing_package_drift");
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", "duration"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ 2s }".into(),
        },
    });

    let package_b = package("module { }\n");
    let (loader_b, entry_b) = composite(&entry, package_b.path());
    let SourceTransactionError::Program(diagnostics) =
        prepare_executable_source_edit_with_loader(&prepared, &batch, &loader_b).unwrap_err()
    else {
        panic!("same package identity with different bytes must be stale");
    };
    assert_eq!(diagnostics.as_slice()[0].code, "PROGRAM_IMPORT_LOAD");
    assert!(diagnostics.as_slice()[0].message.contains("changed bytes"));

    let preview = prepare_executable_source_edit_with_loader(&prepared, &batch, &loader_a)
        .unwrap()
        .execute(&BuildInputManifestV1::empty())
        .unwrap();
    let rebuilt = reprepare_executable_source_edit_preview(&preview, entry_b, &loader_b).unwrap();
    let revision = rebuilt.source_revision().unwrap();
    assert_eq!(
        revision.authored_source_graph_sha256,
        preview.new_revision.authored_source_graph_sha256
    );
    assert_ne!(
        revision.complete_source_graph_sha256,
        preview.new_revision.complete_source_graph_sha256
    );
    assert_ne!(rebuilt.source_graph(), preview.built.source_graph());
}

fn package(source: &str) -> tempfile::TempDir {
    let package = tempfile::tempdir().unwrap();
    write_package_contract(package.path(), source);
    package
}

fn write_package_contract(root: &Path, source: &str) {
    let identity = identity();
    let api = ApiMetadataV1::new(identity.clone(), vec![]);
    let file = LockedFile {
        path: "main.veac".into(),
        sha256: sha256_bytes(source.as_bytes()),
    };
    let manifest = PackageManifestV1::new(
        identity.clone(),
        "main.veac",
        file.sha256.clone(),
        vec![],
        package_api_digest(&api).unwrap(),
    );
    let lock = PackageLockV1::new(identity, vec![file], vec![]).unwrap();
    fs::write(root.join("main.veac"), source).unwrap();
    fs::write(
        root.join("veac.package.api.json"),
        canonical_api_metadata_json(&api).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("veac.package.json"),
        canonical_package_manifest_json(&manifest).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("veac.package.lock"),
        canonical_package_lock_json(&lock).unwrap(),
    )
    .unwrap();
}

fn identity() -> PackageIdentity {
    PackageIdentity {
        name: PackageName::new("root").unwrap(),
        version: ExactVersion::new("1.0.0").unwrap(),
    }
}

fn composite(root: &Path, package: &Path) -> (CompositeSourceLoader, LoadedSource) {
    let (project, entry) = FileSystemLoader::for_entry(root).unwrap();
    let (package, _) = PackageSourceLoader::for_root(package).unwrap();
    let mut loader = CompositeSourceLoader::new(project);
    loader.mount_package(identity(), package).unwrap();
    (loader, entry)
}

fn write_project(temp: &tempfile::TempDir, declarations: &str) -> PathBuf {
    let entry = temp.path().join("main.veac");
    fs::write(&entry, support::project_with(declarations, "duration()")).unwrap();
    entry
}

fn edit(prepared: &veac_lang::program::ExecutableBuild, operation: &str) -> SourceEditBatch {
    SourceEditBatch::new(
        veac_ir::OperationId::new(operation).unwrap(),
        prepared.source_revision().unwrap(),
    )
}
