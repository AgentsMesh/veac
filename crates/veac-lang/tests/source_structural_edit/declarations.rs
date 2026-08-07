use veac_ir::TemporalValue;
use veac_lang::program::{
    apply_executable_source_edit_path, apply_executable_source_edit_path_with_inputs,
    BuildInputBinding, BuildInputManifestV1, BuildInputManifestValue,
};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditOperation, SourceModuleAnchor, SourceNodeRef,
    SourceTemporalProperty,
};

use super::support::{declaration, duration, project_with, Fixture};

#[path = "../executable_authored_temporal/support.rs"]
#[allow(dead_code)]
mod temporal_support;

#[test]
fn set_insert_and_remove_rebuild_constants_and_functions() {
    let entry = project_with(
        "const time base = 1s;\nfn duration(value: time) -> time { value }\nfn obsolete() -> time { 9s }",
        "duration(2s)",
    );
    let fixture = Fixture::new(&entry, &[]);
    let mut batch = fixture.batch("op_declaration_lifecycle");
    batch.operations = vec![
        SourceEditOperation::SetTopLevelDeclaration {
            target: SourceNodeRef::constant("main.veac", "base"),
            declaration: declaration("const time base = 2s;"),
        },
        SourceEditOperation::SetTopLevelDeclaration {
            target: SourceNodeRef::function("main.veac", "duration"),
            declaration: declaration("fn duration(value: time) -> time { value + 1s }"),
        },
        SourceEditOperation::RemoveDeclaration {
            target: SourceNodeRef::function("main.veac", "obsolete"),
        },
        SourceEditOperation::InsertDeclaration {
            module: "main.veac".into(),
            anchor: SourceModuleAnchor::ModuleEnd,
            declaration: declaration("fn retained() -> time { 4s }"),
        },
    ];
    let preview = apply_executable_source_edit_path(&fixture.entry, &batch).unwrap();
    assert_eq!(duration(&preview.built), 3_000);
    let source = preview.source().unwrap();
    assert!(source.contains("const time base = 2s;"));
    assert!(!source.contains(";;"));
    assert!(source.contains("fn retained"));
    assert!(!source.contains("fn obsolete"));
    fixture.assert_unchanged();
}

#[test]
fn exported_constant_replacement_consumes_the_original_semicolon_once() {
    let entry = project_with("import \"./timing.veac\" as timing;", "1s");
    let module = "module {\n  export const time base = 1s;\n}\n";
    let fixture = Fixture::new(&entry, &[("timing.veac", module)]);
    let mut batch = fixture.batch("op_exported_constant_span");
    batch.operations = vec![SourceEditOperation::SetTopLevelDeclaration {
        target: SourceNodeRef::constant("timing.veac", "base"),
        declaration: declaration("export const time base = 2s;"),
    }];

    let preview = apply_executable_source_edit_path(&fixture.entry, &batch).unwrap();
    let edited = preview
        .changes()
        .iter()
        .find(|change| change.module() == "timing.veac")
        .unwrap()
        .source();
    assert_eq!(edited, "module {\n  export const time base = 2s;\n}\n");
    assert!(!edited.contains(";;"));
    assert_eq!(duration(&preview.built), 1_000);
    fixture.assert_unchanged();
}

#[test]
fn nominal_and_implementation_declarations_retype_as_one_batch() {
    let declarations = r#"struct Timing { duration: time, }
enum Padding { Extra, }
impl Timing @timing {
  fn value(self, padding: Padding) -> time {
    match padding { Padding.Extra => self.duration + 1s, }
  }
}
fn duration() -> time { Timing { duration: 1s, }.value(Padding.Extra) }"#;
    let entry = project_with(declarations, "duration()");
    let fixture = Fixture::new(&entry, &[]);
    let mut batch = fixture.batch("op_nominal_top_level_batch");
    batch.operations = vec![
        SourceEditOperation::SetTopLevelDeclaration {
            target: SourceNodeRef::structure("main.veac", "Timing"),
            declaration: declaration("struct Timing { base: time, }"),
        },
        SourceEditOperation::SetTopLevelDeclaration {
            target: SourceNodeRef::enumeration("main.veac", "Padding"),
            declaration: declaration("enum Padding { Double, }"),
        },
        SourceEditOperation::SetTopLevelDeclaration {
            target: SourceNodeRef::implementation("main.veac", "Timing", "timing"),
            declaration: declaration(
                "impl Timing @timing { fn value(self, padding: Padding) -> time { match padding { Padding.Double => self.base + self.base, } } }",
            ),
        },
        SourceEditOperation::SetBody {
            target: SourceNodeRef::function("main.veac", "duration"),
            site: BodySite::FunctionBody,
            body: BodySource {
                source: "{ Timing { base: 1s, }.value(Padding.Double) }".into(),
            },
        },
    ];
    let preview = apply_executable_source_edit_path(&fixture.entry, &batch).unwrap();
    assert_eq!(duration(&preview.built), 2_000);
    assert!(preview.source().unwrap().contains("Padding.Double"));
    fixture.assert_unchanged();
}

#[test]
fn top_level_input_edit_reuses_the_verified_manifest() {
    let entry = project_with("input parameter duration: time;", "duration");
    let fixture = Fixture::new(&entry, &[]);
    let mut inputs = BuildInputManifestV1::empty();
    inputs.inputs.push(BuildInputBinding {
        name: "duration".into(),
        value: BuildInputManifestValue::Time {
            value: "1750ms".into(),
        },
    });
    let mut batch = fixture.batch("op_top_level_input");
    batch.operations = vec![SourceEditOperation::SetTopLevelDeclaration {
        target: SourceNodeRef::input("main.veac", "duration"),
        declaration: declaration("input analysis duration: time;"),
    }];
    let preview =
        apply_executable_source_edit_path_with_inputs(&fixture.entry, &batch, &inputs).unwrap();
    assert_eq!(duration(&preview.built), 1_750);
    assert!(preview
        .source()
        .unwrap()
        .contains("input analysis duration"));
    fixture.assert_unchanged();
}

#[test]
fn top_level_temporal_edit_rebuilds_the_residual_program() {
    let fixture = Fixture::new(temporal_support::MAIN, &[]);
    let mut batch = fixture.batch("op_top_level_temporal");
    batch.operations = vec![SourceEditOperation::SetTopLevelDeclaration {
        target: SourceNodeRef::temporal(
            "main.veac",
            "demo",
            "main",
            "visual",
            "first",
            SourceTemporalProperty::VisualOpacity,
        ),
        declaration: declaration(
            "animate visual-opacity on clip(@demo, @main, @visual, @first) { progress * 0.5 }",
        ),
    }];
    let preview = apply_executable_source_edit_path(&fixture.entry, &batch).unwrap();
    assert_eq!(
        temporal_support::evaluate(
            preview.built.envelope(),
            TemporalValue::Scalar { value: 0.5 }
        ),
        TemporalValue::Scalar { value: 0.25 }
    );
    fixture.assert_unchanged();
}
