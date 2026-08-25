use super::*;
use veac_lang::source_edit::{ImportSource, SourceModuleAnchor};

const FRESH: &str = "module { export fn duration() -> time { 400ms } }\n";

#[test]
fn independent_output_protects_candidate_only_dependencies_and_aliases() {
    for destination in [
        Destination::Direct,
        Destination::HardLink,
        Destination::CaseAlias,
    ] {
        let fixture = Fixture::new(destination);

        veac()
            .args([
                "source-edit",
                fixture.entry.to_str().unwrap(),
                fixture.batch.to_str().unwrap(),
                "--output",
                fixture.output.to_str().unwrap(),
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains("OUTPUT_OVERWRITES_INPUT"));

        assert_eq!(std::fs::read_to_string(&fixture.fresh).unwrap(), FRESH);
        assert_eq!(
            std::fs::read_to_string(&fixture.entry).unwrap(),
            fixture.source
        );
    }
}

#[derive(Clone, Copy)]
enum Destination {
    Direct,
    HardLink,
    CaseAlias,
}

struct Fixture {
    _temp: tempfile::TempDir,
    entry: std::path::PathBuf,
    fresh: std::path::PathBuf,
    batch: std::path::PathBuf,
    output: std::path::PathBuf,
    source: String,
}

impl Fixture {
    fn new(destination: Destination) -> Self {
        let temp = tempdir().unwrap();
        let source = format!(
            "fn duration() -> time {{ 200ms }}\n{}",
            GENERATED_SOURCE.replace("during(0s, 200ms)", "during(0s, duration())")
        );
        let entry = source_file(&temp, &source);
        let fresh = temp.path().join("fresh.veac");
        std::fs::write(&fresh, FRESH).unwrap();
        let batch = write_batch(&entry, temp.path());
        let output = match destination {
            Destination::Direct => fresh.clone(),
            Destination::HardLink => {
                let alias = temp.path().join("fresh-alias.veac");
                std::fs::hard_link(&fresh, &alias).unwrap();
                alias
            }
            Destination::CaseAlias => temp.path().join("FRESH.veac"),
        };
        Self {
            _temp: temp,
            entry,
            fresh,
            batch,
            output,
            source,
        }
    }
}

fn write_batch(entry: &std::path::Path, root: &std::path::Path) -> std::path::PathBuf {
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_candidate_output_guard").unwrap(),
        revision(entry),
    );
    batch.operations = vec![
        SourceEditOperation::InsertImport {
            module: "main.veac".into(),
            anchor: SourceModuleAnchor::ModuleStart,
            import: ImportSource {
                path: "./fresh.veac".into(),
                alias: "fresh".into(),
            },
        },
        SourceEditOperation::SetBody {
            target: SourceNodeRef::function("main.veac", "duration"),
            site: BodySite::FunctionBody,
            body: BodySource {
                source: "{ fresh.duration() }".into(),
            },
        },
    ];
    let path = root.join("candidate-output-edit.json");
    std::fs::write(
        &path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&batch).unwrap(),
    )
    .unwrap();
    path
}
