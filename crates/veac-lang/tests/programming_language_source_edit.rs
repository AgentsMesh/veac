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
    assert_eq!(
        &batch.base_revision,
        prepared.source_index().unwrap().revision()
    );
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
        preview.new_revision.source_graph_sha256,
        "9e728b823124e0b4592b70d4b107c11050982bfea3fdb29c9263e30ccc12574d"
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
