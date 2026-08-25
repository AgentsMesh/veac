use std::fs;

use tempfile::tempdir;
use veac_lang::program::{apply_executable_source_edit_path, build_path};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef, SourcePrecondition,
};

#[path = "program_functions/support.rs"]
mod support;

const DECLARATIONS: &str = r#"struct Timing { start: time, duration: time, }
impl Timing @timing {
  fn finish(self) -> time { self.start + self.duration }
}
fn render(value: Timing) -> time { value.finish() }"#;

#[test]
fn editing_a_method_body_retypes_and_reexecutes_from_source() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let source = support::project_with(DECLARATIONS, "render(Timing { duration: 2s, start: 1s, })");
    fs::write(&entry, &source).unwrap();
    let built = build_path(&entry).unwrap();
    assert_eq!(support::result_duration(&built), "3s");

    let index = built.source_index().unwrap();
    let target = SourceNodeRef::method("main.veac", "Timing", "finish");
    let original = BodySource {
        source: "{ self.start + self.duration }".into(),
    };
    assert_eq!(
        index.body(&target, BodySite::MethodBody).unwrap().source,
        original.source
    );
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_method_body_edit").unwrap(),
        built.source_revision().unwrap(),
    );
    batch.preconditions.push(SourcePrecondition::BodyEquals {
        target: target.clone(),
        site: BodySite::MethodBody,
        body: original,
    });
    batch.operations.push(SourceEditOperation::SetBody {
        target,
        site: BodySite::MethodBody,
        body: BodySource {
            source: "{ self.duration }".into(),
        },
    });

    let preview = apply_executable_source_edit_path(&entry, &batch).unwrap();
    assert_eq!(support::result_duration(&preview.built), "2s");
    assert!(preview.source().unwrap().contains("{ self.duration }"));
    assert_eq!(fs::read_to_string(&entry).unwrap(), source);
    support::validate_canonical(&preview.built);
}
