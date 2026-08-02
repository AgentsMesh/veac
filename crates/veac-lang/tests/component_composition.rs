use std::fs;

use tempfile::tempdir;
use veac_lang::authoring::SourceDecl;
use veac_lang::program::{apply_source_edit_path, compile_path, compile_source};
use veac_lang::source_edit::{
    ExpressionSite, ExpressionSource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

#[path = "component_composition/fixtures.rs"]
mod fixtures;
use fixtures::{COMPOSED, ENTRY, MODULE};

#[test]
fn nested_component_lowers_slots_parameters_and_sequence_references_to_core() {
    let compiled = compile_source(COMPOSED).unwrap();
    let child_id = "veac-h-4-root-4-card";
    let child = sequence(compiled.document(), child_id);
    assert_eq!(child.layers[0].items[0].record.duration.raw, "2s");
    assert!(matches!(
        child.layers[0].items[0].source,
        SourceDecl::Generated { .. }
    ));
    let parent = sequence(compiled.document(), "root");
    let SourceDecl::Sequence { sequence, .. } = &parent.layers[0].items[0].source else {
        panic!("parent must reference the expanded child sequence");
    };
    assert_eq!(sequence.id.value, child_id);
    assert_eq!(
        compiled
            .provenance()
            .get_local("root", "card")
            .unwrap()
            .path,
        "main.veac"
    );
    let leaf = compiled
        .provenance()
        .get_local_path("root", &["card", "leaf"])
        .unwrap();
    assert_eq!(leaf.definition.as_deref(), Some("leaf"));
    veac_ir::validate(&veac_lang::authoring::lower_document(compiled.document()).unwrap()).unwrap();
}

#[test]
fn child_defaults_use_definition_scope_and_binds_use_parent_instance_scope() {
    let (temp, entry) = fixture();
    let compiled = compile_path(&entry).unwrap();
    assert_eq!(child_duration(&compiled, "default-leaf"), "1s");
    assert_eq!(child_duration(&compiled, "bound-leaf"), "5s");
    assert_eq!(
        compiled
            .provenance()
            .get_local("root", "bound-leaf")
            .unwrap()
            .path,
        "components.veac"
    );
    drop(temp);
}

#[test]
fn source_edit_of_parent_binding_recompiles_nested_ir_and_provenance() {
    let (temp, entry) = fixture();
    let compiled = compile_path(&entry).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_nested_component").unwrap(),
        compiled.source_index().unwrap().revision().clone(),
    );
    batch.operations.push(SourceEditOperation::SetExpression {
        target: SourceNodeRef::component_instance("main.veac", "root"),
        site: ExpressionSite::ComponentInstanceArgument {
            parameter: "duration".into(),
        },
        expression: ExpressionSource {
            source: "7s".into(),
        },
    });
    let preview = apply_source_edit_path(&entry, &batch).unwrap();
    assert_eq!(child_duration(&preview.compiled, "bound-leaf"), "7s");
    assert_eq!(child_duration(&preview.compiled, "default-leaf"), "1s");
    assert_eq!(fs::read_to_string(&entry).unwrap(), ENTRY);
    assert!(preview
        .compiled
        .provenance()
        .get_local_path("root", &["bound-leaf", "content"])
        .is_some());
    drop(temp);
}

#[test]
fn recursive_component_graphs_fail_with_a_stable_cycle_diagnostic() {
    let declarations = r#"component sequence first {
  instance sequence @next from second {} body {}
}
component sequence second {
  instance sequence @next from first {} body {}
}"#;
    let source = format!("{declarations}\n{}", project("cycle"));
    let error = compile_source(&source).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_COMPONENT_CYCLE");
    assert!(error.as_slice()[0].message.contains("first"));
    assert!(error.as_slice()[0].message.contains("second"));
}

#[test]
fn deep_parameterless_instances_do_not_materialize_captured_text_per_frame() {
    let components = (0..16)
        .map(|level| {
            if level == 0 {
                "component sequence level-0 { body {} }".to_owned()
            } else {
                format!(
                    "component sequence level-{level} {{\n  \
                     instance sequence @c from level-{} {{}}\n  body {{}}\n}}",
                    level - 1
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let captured = "x".repeat(256 * 1024);
    let source = format!(
        "const text large = \"{captured}\";\n{components}\n\
         instance sequence root from level-15 {{}}\n{}",
        project("deep-shared-values")
    );
    let compiled = compile_source(&source).unwrap();
    let path = vec!["c"; 15];
    assert!(compiled
        .provenance()
        .get_local_path("root", &path)
        .is_some());
}

#[test]
fn nested_sequences_share_the_final_expanded_source_budget() {
    let children = (0..9)
        .map(|value| format!("instance sequence @child-{value} from leaf {{}}"))
        .collect::<Vec<_>>()
        .join("\n");
    let filler = "x".repeat(4 * 1024 * 1024);
    let declarations = format!(
        "component sequence leaf {{ body {{ /*{filler}*/ }} }}\n\
         component sequence shell {{ {children} body {{}} }}"
    );
    let source = format!("{declarations}\n{}", project("budget"));
    let error = compile_source(&source).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_EXPANSION_BUDGET");
}

fn child_duration(compiled: &veac_lang::program::CompiledProgram, local: &str) -> String {
    let id = veac_hygienic("root", local);
    sequence(compiled.document(), &id).layers[0].items[0]
        .record
        .duration
        .raw
        .clone()
}

fn sequence<'a>(
    document: &'a veac_lang::authoring::Document,
    id: &str,
) -> &'a veac_lang::authoring::SequenceDecl {
    document
        .project
        .sequences
        .iter()
        .find(|value| value.id.value == id)
        .unwrap()
}

fn veac_hygienic(root: &str, local: &str) -> String {
    format!("veac-h-{}-{root}-{}-{local}", root.len(), local.len())
}

fn fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, ENTRY).unwrap();
    fs::write(temp.path().join("components.veac"), MODULE).unwrap();
    (temp, entry)
}

fn project(id: &str) -> String {
    format!(
        r#"project {id} {{
  settings {{ timebase 1/1000; canvas 1px by 1px; frame-rate 1fps; sample-rate 8000hz; }}
  entry sequence main; sequence main {{}}
}}"#
    )
}
