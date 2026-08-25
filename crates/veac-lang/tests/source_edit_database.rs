use std::fs;

use tempfile::tempdir;
use veac_lang::program::{
    BuildInputManifestV1, CompilerDatabase, LoadedSource, SourceTransactionError,
};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

#[path = "source_edit_database/interface.rs"]
mod interface;
#[path = "source_edit_database/support.rs"]
mod loader_support;
#[path = "program_functions/support.rs"]
mod support;

const MODULE: &str = "module { export fn imported() -> time { 1s } }\n";
const STABLE: &str = "module { export fn stable() -> time { 2s } }\n";
const CHAIN: &str =
    "module { import \"./leaf.veac\" as leaf; export fn imported() -> time { leaf.value() } }\n";
const LEAF: &str = "module { export fn value() -> time { 1s } }\n";

#[test]
fn database_backed_edit_reuses_unchanged_queries_and_matches_clean_build() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let module = temp.path().join("timing.veac");
    let stable = temp.path().join("stable.veac");
    fs::write(
        &entry,
        support::project_with(
            "import \"./timing.veac\" as timing;\nimport \"./stable.veac\" as stable;",
            "timing.imported()",
        ),
    )
    .unwrap();
    fs::write(&module, MODULE).unwrap();
    fs::write(&stable, STABLE).unwrap();
    let database = CompilerDatabase::default();
    let prepared = database.prepare_path(&entry).unwrap();
    let before = database.statistics();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_incremental_edit").unwrap(),
        prepared.source_revision().unwrap(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("timing.veac", "imported"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ 2500ms }".into(),
        },
    });
    let (_, candidate) = database.prepare_source_edit_path(&entry, &batch).unwrap();
    let direct = candidate.program().execute().unwrap();
    let direct_json = veac_ir::canonical_json(direct.envelope()).unwrap();
    let before_consuming_execute = database.statistics();
    let preview = candidate.execute(&BuildInputManifestV1::empty()).unwrap();
    assert_eq!(
        direct_json,
        veac_ir::canonical_json(preview.built.envelope()).unwrap()
    );
    assert_eq!(database.statistics(), before_consuming_execute);
    let after = database.statistics();
    assert_eq!(after.syntax_hits - before.syntax_hits, 5);
    assert_eq!(after.syntax_misses - before.syntax_misses, 1);
    assert_eq!(after.hir_hits - before.hir_hits, 4);
    assert_eq!(after.core_hits - before.core_hits, 4);
    assert_eq!(after.semantic_invalidations, 1);
    assert_eq!(preview.built.envelope().project.sequences.len(), 1);
    let clean_source = fs::read_to_string(&entry).unwrap();
    let edited_module = preview.changes()[0].source().to_owned();
    let clean = CompilerDatabase::default()
        .prepare_with_loader(
            veac_lang::program::LoadedSource {
                id: "main.veac".into(),
                source: clean_source,
            },
            &OverlayLoader {
                timing: edited_module,
            },
        )
        .unwrap()
        .execute()
        .unwrap();
    assert_eq!(
        veac_ir::canonical_json(preview.built.envelope()).unwrap(),
        veac_ir::canonical_json(clean.envelope()).unwrap()
    );
    let compiled = database.statistics();
    preview.built.envelope();
    assert_eq!(database.statistics(), compiled);
}

struct OverlayLoader {
    timing: String,
}

impl veac_lang::program::SourceLoader for OverlayLoader {
    fn load(
        &self,
        _importer: &str,
        requested: &str,
    ) -> Result<veac_lang::program::LoadedSource, String> {
        let (id, source) = match requested {
            "./timing.veac" => ("timing.veac", self.timing.as_str()),
            "./stable.veac" => ("stable.veac", STABLE),
            _ => return Err(format!("missing module {requested}")),
        };
        Ok(veac_lang::program::LoadedSource {
            id: id.into(),
            source: source.into(),
        })
    }

    fn authority(&self, _source_id: &str) -> veac_lang::program::SourceAuthority {
        veac_lang::program::SourceAuthority::Project
    }
}

#[test]
fn database_edit_keeps_sources_atomic_when_candidate_fails() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, support::project_with("", "1s")).unwrap();
    let original = fs::read_to_string(&entry).unwrap();
    let database = CompilerDatabase::default();
    let prepared = database.prepare_path(&entry).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_failed_edit").unwrap(),
        prepared.source_revision().unwrap(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", "main"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ missing() }".into(),
        },
    });
    let error = database
        .prepare_source_edit_path(&entry, &batch)
        .and_then(|candidate| candidate.1.execute(&BuildInputManifestV1::empty()))
        .unwrap_err();
    assert!(matches!(error, SourceTransactionError::Program(_)));
    assert_eq!(fs::read_to_string(entry).unwrap(), original);
}

#[test]
fn changed_leaf_invalidates_transitive_importers_and_reuses_stable_module() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let source = support::project_with(
        "import \"./timing.veac\" as timing;\nimport \"./stable.veac\" as stable;",
        "timing.imported()",
    );
    fs::write(&entry, &source).unwrap();
    let loader = loader_support::MutableLoader::new([
        ("timing.veac", CHAIN),
        ("leaf.veac", LEAF),
        ("stable.veac", STABLE),
    ]);
    let database = CompilerDatabase::default();
    database
        .prepare_with_loader(
            LoadedSource {
                id: "main.veac".into(),
                source: source.clone(),
            },
            &loader,
        )
        .unwrap();
    let before = database.statistics();
    loader.replace("leaf.veac", "module { export fn value() -> time { 3s } }\n");
    database
        .prepare_with_loader(
            LoadedSource {
                id: "main.veac".into(),
                source,
            },
            &loader,
        )
        .unwrap();
    let after = database.statistics();
    assert_eq!(after.semantic_invalidations, 2);
    assert!(after.syntax_hits - before.syntax_hits >= 3);
    assert!(after.hir_hits - before.hir_hits >= 1);
    assert!(after.core_hits - before.core_hits >= 1);
    assert_eq!(after.pending_semantic_invalidations, 0);
}
