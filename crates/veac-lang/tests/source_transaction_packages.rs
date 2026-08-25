use std::path::Path;

use veac_lang::package::{ExactVersion, PackageIdentity, PackageName, PackageSourceLoader};
use veac_lang::program::{
    prepare_executable_source_edit_with_loader, prepare_with_loader, CompositeSourceLoader,
    FileSystemLoader, LoadedSource, SourceAuthority, SourceLoader, SourceTransactionError,
};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

#[path = "program_functions/support.rs"]
mod support;

const PACKAGE: &str = "veac-components";
const VERSION: &str = "0.1.0";

fn package_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib/veac-components")
}

fn identity() -> PackageIdentity {
    PackageIdentity {
        name: PackageName::new(PACKAGE).unwrap(),
        version: ExactVersion::new(VERSION).unwrap(),
    }
}

fn loader(temp: &tempfile::TempDir) -> (CompositeSourceLoader, veac_lang::program::LoadedSource) {
    let source = support::project_with(
        "import \"package:veac-components@0.1.0/main.veac\" as components;\n\
         fn duration() -> time { 1s }",
        "duration()",
    );
    let entry_path = temp.path().join("main.veac");
    std::fs::write(&entry_path, &source).unwrap();
    let (project, entry) = FileSystemLoader::for_entry(&entry_path).unwrap();
    let (package, _) = PackageSourceLoader::for_root(&package_root()).unwrap();
    let mut composite = CompositeSourceLoader::new(project);
    composite.mount_package(identity(), package).unwrap();
    (composite, entry)
}

fn batch(
    program: &veac_lang::program::ExecutableBuild,
    target: SourceNodeRef,
    body: &str,
) -> SourceEditBatch {
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_package_source_edit_test").unwrap(),
        program.source_revision().unwrap(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target,
        site: BodySite::FunctionBody,
        body: BodySource {
            source: body.into(),
        },
    });
    batch
}

#[test]
fn composite_loader_assigns_project_and_dependency_authority() {
    let temp = tempfile::tempdir().unwrap();
    let (loader, entry) = loader(&temp);
    let prepared = prepare_with_loader(entry, &loader).unwrap();
    let (project, _) = FileSystemLoader::for_entry(&temp.path().join("main.veac")).unwrap();
    let (package, _) = PackageSourceLoader::for_root(&package_root()).unwrap();
    assert_eq!(project.authority("main.veac"), SourceAuthority::Project);
    assert_eq!(
        package.authority("main.veac"),
        SourceAuthority::ReadOnlyDependency
    );
    assert_eq!(loader.authority("main.veac"), SourceAuthority::Project);
    assert_eq!(
        loader.authority("packages/veac-components@0.1.0/main.veac"),
        SourceAuthority::ReadOnlyDependency
    );
    assert!(prepared
        .sources()
        .contains_key("packages/veac-components@0.1.0/main.veac"));
}

#[test]
fn source_edit_rejects_a_package_module_before_overlaying_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let (loader, entry) = loader(&temp);
    let prepared = prepare_with_loader(entry, &loader).unwrap();
    let target = SourceNodeRef::function("packages/veac-components@0.1.0/main.veac", "card");
    let batch = batch(&prepared, target, "{ 1s }");
    let error = prepare_executable_source_edit_with_loader(&prepared, &batch, &loader).unwrap_err();
    assert!(matches!(
        error,
        SourceTransactionError::ReadOnlySource { .. }
    ));
}

#[test]
fn candidate_reloads_unchanged_package_sources_through_the_fallback_loader() {
    let temp = tempfile::tempdir().unwrap();
    let (loader, entry) = loader(&temp);
    let prepared = prepare_with_loader(entry, &loader).unwrap();
    let target = SourceNodeRef::function("main.veac", "duration");
    let batch = batch(&prepared, target, "{ 2s }");
    let candidate = prepare_executable_source_edit_with_loader(&prepared, &batch, &loader).unwrap();
    assert!(candidate
        .program()
        .sources()
        .contains_key("packages/veac-components@0.1.0/main.veac"));
    let preview = candidate
        .execute(&veac_lang::program::BuildInputManifestV1::empty())
        .unwrap();
    assert_eq!(preview.previous_modules(), ["main.veac"]);
    assert_eq!(
        preview.previous_sources().keys().collect::<Vec<_>>(),
        ["main.veac"]
    );
    assert!(preview
        .built
        .sources()
        .contains_key("packages/veac-components@0.1.0/main.veac"));
}

struct ReauthorizingLoader<'a>(&'a dyn SourceLoader);

impl SourceLoader for ReauthorizingLoader<'_> {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String> {
        self.0.load(importer, requested)
    }

    fn authority(&self, _source_id: &str) -> SourceAuthority {
        SourceAuthority::Project
    }
}

#[test]
fn edit_loader_cannot_reauthorize_a_prepared_dependency() {
    let temp = tempfile::tempdir().unwrap();
    let (loader, entry) = loader(&temp);
    let prepared = prepare_with_loader(entry, &loader).unwrap();
    let target = SourceNodeRef::function("packages/veac-components@0.1.0/main.veac", "card");
    let batch = batch(&prepared, target, "{ 1s }");
    let error = prepare_executable_source_edit_with_loader(
        &prepared,
        &batch,
        &ReauthorizingLoader(&loader),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        SourceTransactionError::ReadOnlySource { .. }
    ));
}

struct ChangedBytesLoader<'a>(&'a dyn SourceLoader);

impl SourceLoader for ChangedBytesLoader<'_> {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String> {
        let mut loaded = self.0.load(importer, requested)?;
        if loaded.id.starts_with("packages/") {
            loaded.source.push('\n');
        }
        Ok(loaded)
    }

    fn authority(&self, source_id: &str) -> SourceAuthority {
        self.0.authority(source_id)
    }
}

#[test]
fn edit_loader_cannot_replace_prepared_dependency_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let (loader, entry) = loader(&temp);
    let prepared = prepare_with_loader(entry, &loader).unwrap();
    let target = SourceNodeRef::function("main.veac", "duration");
    let batch = batch(&prepared, target, "{ 2s }");
    let error =
        prepare_executable_source_edit_with_loader(&prepared, &batch, &ChangedBytesLoader(&loader))
            .unwrap_err();
    let SourceTransactionError::Program(diagnostics) = error else {
        panic!("changed dependency bytes must fail source preparation");
    };
    assert_eq!(diagnostics.as_slice()[0].code, "PROGRAM_IMPORT_LOAD");
    assert!(diagnostics.as_slice()[0].message.contains("changed bytes"));
}
