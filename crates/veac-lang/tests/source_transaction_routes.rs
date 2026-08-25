use veac_lang::program::{
    prepare_executable_source_edit_with_loader, prepare_with_loader, LoadedSource, SourceAuthority,
    SourceLoader, SourceTransactionError,
};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

#[path = "program_functions/support.rs"]
mod support;

const ACTUAL: &str = "module { export fn value() -> time { 1s } }\n";
const DECOY: &str = "module { export fn value() -> time { 9s } }\n";

struct AliasingLoader;

impl SourceLoader for AliasingLoader {
    fn load(&self, _: &str, requested: &str) -> Result<LoadedSource, String> {
        match requested {
            "./dep.veac" => Ok(LoadedSource {
                id: "actual.veac".into(),
                source: ACTUAL.into(),
            }),
            "./other.veac" => Ok(LoadedSource {
                id: "dep.veac".into(),
                source: DECOY.into(),
            }),
            _ => Err(format!("missing module `{requested}`")),
        }
    }

    fn authority(&self, _: &str) -> SourceAuthority {
        SourceAuthority::Project
    }
}

struct RetargetingLoader;

impl SourceLoader for RetargetingLoader {
    fn load(&self, _: &str, requested: &str) -> Result<LoadedSource, String> {
        match requested {
            "./dep.veac" | "./other.veac" => Ok(LoadedSource {
                id: "dep.veac".into(),
                source: DECOY.into(),
            }),
            _ => Err(format!("missing module `{requested}`")),
        }
    }

    fn authority(&self, _: &str) -> SourceAuthority {
        SourceAuthority::Project
    }
}

#[test]
fn candidate_edits_preserve_a_frozen_non_path_route() {
    let prepared = prepared();
    let batch = batch(&prepared);
    let preview = prepare_executable_source_edit_with_loader(&prepared, &batch, &AliasingLoader)
        .unwrap()
        .execute(&veac_lang::program::BuildInputManifestV1::empty())
        .unwrap();
    assert_eq!(support::result_duration(&preview.built), "2s");
    assert_eq!(preview.built.sources()["actual.veac"], ACTUAL);
    assert_eq!(preview.built.sources()["dep.veac"], DECOY);
}

#[test]
fn candidate_rejects_a_changed_frozen_non_path_route() {
    let prepared = prepared();
    let batch = batch(&prepared);
    let SourceTransactionError::Program(diagnostics) =
        prepare_executable_source_edit_with_loader(&prepared, &batch, &RetargetingLoader)
            .unwrap_err()
    else {
        panic!("candidate must reject a changed canonical route");
    };
    assert!(diagnostics.as_slice()[0]
        .message
        .contains("changed source ID from `actual.veac` to `dep.veac`"));
}

fn prepared() -> veac_lang::program::ExecutableBuild {
    let source = support::project_with(
        "import \"./dep.veac\" as dep;\n\
         import \"./other.veac\" as other;\n\
         fn duration() -> time { dep.value() }",
        "duration()",
    );
    prepare_with_loader(
        LoadedSource {
            id: "main.veac".into(),
            source,
        },
        &AliasingLoader,
    )
    .unwrap()
}

fn batch(prepared: &veac_lang::program::ExecutableBuild) -> SourceEditBatch {
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_frozen_non_path_route").unwrap(),
        prepared.source_revision().unwrap(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", "duration"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ dep.value() + 1s }".into(),
        },
    });
    batch
}
