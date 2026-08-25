use std::fs;
use std::path::Path;

use veac_lang::program::{
    apply_executable_source_edit_path_with_inputs, parse_build_input_manifest, prepare_path,
};
use veac_lang::source_edit::{
    decode_source_edit_batch_json, SourceEditOperation, SourcePrecondition,
    SOURCE_EDIT_SCHEMA_VERSION,
};

#[test]
fn checked_in_function_edit_is_typed_revision_bound_and_executable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let entry = root.join("examples/programming-language/main.veac");
    let source = fs::read_to_string(&entry).unwrap();
    let json =
        fs::read_to_string(root.join("examples/programming-language/source-edit.json")).unwrap();
    let inputs = parse_build_input_manifest(
        &fs::read_to_string(root.join("examples/programming-language/build-inputs.json")).unwrap(),
    )
    .unwrap();
    let batch = decode_source_edit_batch_json(&json).unwrap();
    let prepared = prepare_path(&entry).unwrap();
    assert_eq!(batch.schema_version, SOURCE_EDIT_SCHEMA_VERSION);
    assert_eq!(batch.base_revision, prepared.source_revision().unwrap());
    assert!(matches!(
        batch.preconditions.as_slice(),
        [
            SourcePrecondition::ImportExists { .. },
            SourcePrecondition::NodeExists { .. },
            SourcePrecondition::StatementEquals { .. }
        ]
    ));
    assert!(matches!(
        batch.operations.as_slice(),
        [
            SourceEditOperation::InsertImport { .. },
            SourceEditOperation::RemoveImport { .. },
            SourceEditOperation::InsertDeclaration { .. },
            SourceEditOperation::RemoveDeclaration { .. },
            SourceEditOperation::SetStatement { .. }
        ]
    ));

    let preview = apply_executable_source_edit_path_with_inputs(&entry, &batch, &inputs).unwrap();
    assert_eq!(preview.changed_modules(), ["brand.veac", "main.veac"]);
    assert_eq!(
        preview.new_revision.authored_source_graph_sha256,
        "4fbe74e49426b74f334eacf913c6c7edd27ab934f2a9ece1b01077e6655896d8"
    );
    assert_eq!(
        preview.new_revision.complete_source_graph_sha256,
        "f5a3880f209a15fff8cc9634454398b5c23b5cdf44dd7b54fa6e064fd61d690a"
    );
    let sequence = &preview.built.envelope().project.sequences[0];
    assert_eq!(sequence.tracks.len(), 2);
    assert!(sequence.tracks.iter().all(|track| track.clips.len() == 2));
    assert_eq!(sequence.tracks[0].clips[1].record_range.start.value, 1800);
    assert!(preview
        .changes()
        .iter()
        .find(|change| change.module() == "main.veac")
        .unwrap()
        .source()
        .contains("let span = brand.card_span(base, 600ms);"));
    assert_eq!(fs::read_to_string(entry).unwrap(), source);
}
