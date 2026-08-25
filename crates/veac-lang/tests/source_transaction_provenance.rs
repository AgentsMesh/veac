use veac_lang::program::{
    prepare_executable_source_edit_with_loader, prepare_with_loader, LoadedSource, SourceAuthority,
    SourceLoader, SourceTransactionError,
};
use veac_lang::source_edit::{
    source_graph_revision, BodySite, BodySource, SourceEditBatch, SourceEditOperation,
    SourceModule, SourceNodeRef,
};

#[path = "program_functions/support.rs"]
mod support;

const DEPENDENCY: &str = "module { export fn value() -> time { 1s } }\n";
const CHANGED_DEPENDENCY: &str = "module { export fn value() -> time { 9s } }\n";

struct DependencyLoader {
    source: &'static str,
    authority: SourceAuthority,
}

impl SourceLoader for DependencyLoader {
    fn load(&self, _: &str, requested: &str) -> Result<LoadedSource, String> {
        (requested == "./dependency.veac")
            .then(|| LoadedSource {
                id: "dependency.veac".into(),
                source: self.source.into(),
            })
            .ok_or_else(|| format!("missing module `{requested}`"))
    }

    fn authority(&self, source_id: &str) -> SourceAuthority {
        if source_id == "dependency.veac" {
            self.authority
        } else {
            SourceAuthority::Project
        }
    }
}

fn source() -> String {
    support::project_with(
        "import \"./dependency.veac\" as dependency;\nfn duration() -> time { dependency.value() }",
        "duration()",
    )
}

fn prepared() -> veac_lang::program::ExecutableBuild {
    prepared_with_dependency(DEPENDENCY)
}

fn prepared_with_dependency(dependency: &'static str) -> veac_lang::program::ExecutableBuild {
    prepare_with_loader(
        LoadedSource {
            id: "main.veac".into(),
            source: source(),
        },
        &DependencyLoader {
            source: dependency,
            authority: SourceAuthority::ReadOnlyDependency,
        },
    )
    .unwrap()
}

fn batch(
    prepared: &veac_lang::program::ExecutableBuild,
    module: &str,
    function: &str,
) -> SourceEditBatch {
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_prepared_provenance").unwrap(),
        prepared.source_revision().unwrap(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function(module, function),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ 2s }".into(),
        },
    });
    batch
}

#[test]
fn edit_loader_cannot_reauthorize_a_prepared_read_only_source() {
    let prepared = prepared();
    let batch = batch(&prepared, "dependency.veac", "value");
    let permissive = DependencyLoader {
        source: DEPENDENCY,
        authority: SourceAuthority::Project,
    };
    let error =
        prepare_executable_source_edit_with_loader(&prepared, &batch, &permissive).unwrap_err();
    assert!(matches!(
        error,
        SourceTransactionError::ReadOnlySource { module }
            if module == "dependency.veac"
    ));
}

#[test]
fn candidate_rejects_changed_bytes_for_a_prepared_dependency() {
    let prepared = prepared();
    let batch = batch(&prepared, "main.veac", "duration");
    let changed = DependencyLoader {
        source: CHANGED_DEPENDENCY,
        authority: SourceAuthority::Project,
    };
    let SourceTransactionError::Program(error) =
        prepare_executable_source_edit_with_loader(&prepared, &batch, &changed).unwrap_err()
    else {
        panic!("changed dependency bytes must fail candidate preparation");
    };
    assert_eq!(error.as_slice()[0].code, "PROGRAM_IMPORT_LOAD");
    assert!(error.as_slice()[0]
        .message
        .contains("changed bytes since the prepared graph"));
}

#[test]
fn public_source_index_excludes_frozen_dependencies() {
    let prepared = prepared();
    let inventory = prepared.source_inventory().unwrap();
    assert_eq!(
        inventory
            .modules
            .iter()
            .map(|module| module.module.as_str())
            .collect::<Vec<_>>(),
        ["main.veac"]
    );
    assert!(prepared.sources().contains_key("dependency.veac"));

    let built = prepared.execute().unwrap();
    assert_eq!(built.source_inventory().unwrap(), inventory);
    assert!(built.sources().contains_key("dependency.veac"));
    assert_eq!(
        built.source_graph().authority("dependency.veac"),
        SourceAuthority::ReadOnlyDependency
    );
    assert_eq!(
        built.source_graph().complete_revision(),
        prepared.source_graph().complete_revision()
    );
}

#[test]
fn source_index_revision_is_bound_to_its_complete_frozen_graph() {
    let first = prepared();
    let second = prepared_with_dependency(CHANGED_DEPENDENCY);
    let first_revision = first.source_revision().unwrap();
    let second_revision = second.source_revision().unwrap();
    assert_eq!(
        first_revision.authored_source_graph_sha256,
        second_revision.authored_source_graph_sha256
    );
    assert_ne!(
        first_revision.complete_source_graph_sha256,
        second_revision.complete_source_graph_sha256
    );
    assert!(matches!(
        first.source_index().unwrap().inventory(&second_revision),
        Err(veac_lang::source_edit::SourceEditError::StaleRevision { .. })
    ));
}

#[test]
fn candidate_revision_tracks_only_authored_sources() {
    let prepared = prepared();
    let batch = batch(&prepared, "main.veac", "duration");
    let preview = prepare_executable_source_edit_with_loader(
        &prepared,
        &batch,
        &DependencyLoader {
            source: DEPENDENCY,
            authority: SourceAuthority::ReadOnlyDependency,
        },
    )
    .unwrap()
    .execute(&veac_lang::program::BuildInputManifestV1::empty())
    .unwrap();
    let main = preview.built.sources().get("main.veac").unwrap();
    let expected = source_graph_revision(&[SourceModule::utf8("main.veac", main)]).unwrap();
    assert_eq!(
        preview.new_revision.authored_source_graph_sha256,
        expected.source_graph_sha256
    );
    assert_eq!(preview.built.source_index().unwrap().revision(), &expected);
}
