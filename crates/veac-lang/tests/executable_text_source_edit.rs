use std::fs;

use tempfile::{tempdir, TempDir};
use veac_ir::ClipSource;
use veac_lang::program::{
    apply_executable_source_edit_path, build_path, prepare_path, SourceTransactionError,
};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

const SOURCE: &str = include_str!("../../../examples/executable-text-family/main.veac");
const PREVIEW: &str = include_str!("../../../examples/executable-text-family/preview.veac");
const FONT: &[u8] =
    include_bytes!("../../../examples/executable-text-family/assets/veac-example-zh.ttf");

const EDITED_BODY: &str = r#"{
  item(
    key, item_enabled(), during(start, 2s),
    source_caption_speaker(content, "小石", vertical_style(
      fonts, text_line(), writing_vertical_lr(), orientation_mixed()
    )),
    source_timing_native()
  )
}"#;

const TEMPLATE_BODY: &str = r#"{
  item(
    key, item_enabled(), during(start, 2s),
    source_text(content, style(fonts, granularity)), source_timing_native()
  ).with_template(template_contract(
    slot_text(), fill_fit_duration(), "可编辑标题",
    source_duration_any(), template_text_editable()
  ))
}"#;

#[test]
fn indexed_text_function_edit_rebuilds_canonical_preview_without_writing_source() {
    let fixture = fixture();
    let before = fs::read(&fixture.entry).unwrap();
    let prepared = prepare_path(&fixture.entry).unwrap();
    let target = SourceNodeRef::function("main.veac", "speaker_for");
    assert!(prepared
        .source_index()
        .unwrap()
        .body(&target, BodySite::FunctionBody)
        .is_some());

    let preview = apply_executable_source_edit_path(
        &fixture.entry,
        &batch(
            &fixture.entry,
            "speaker_for",
            EDITED_BODY,
            "op_text_preview",
        ),
    )
    .unwrap();
    let caption = preview.built.envelope().project.sequences[0]
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .find(|clip| {
            matches!(
                &clip.source,
                ClipSource::Caption {
                    speaker: Some(speaker),
                    ..
                } if speaker == "小石"
            )
        })
        .expect("edited caption speaker");
    assert!(matches!(caption.source, ClipSource::Caption { .. }));
    assert!(preview.source().unwrap().contains("小石"));
    assert_eq!(fs::read(&fixture.entry).unwrap(), before);
    let json = veac_ir::canonical_json(preview.built.envelope()).unwrap();
    assert_eq!(
        veac_ir::decode_canonical_json(&json).unwrap(),
        *preview.built.envelope()
    );
}

#[test]
fn invalid_text_edit_is_atomic_against_source_and_canonical_baseline() {
    let fixture = fixture();
    let before = fs::read(&fixture.entry).unwrap();
    let baseline = veac_ir::canonical_json(build_path(&fixture.entry).unwrap().envelope()).unwrap();
    let invalid = EDITED_BODY.replace("\"小石\"", "\"   \"");
    let error = apply_executable_source_edit_path(
        &fixture.entry,
        &batch(&fixture.entry, "speaker_for", &invalid, "op_text_invalid"),
    )
    .unwrap_err();
    assert!(matches!(error, SourceTransactionError::Program(_)));
    assert!(error.to_string().contains("EXECUTABLE_LOWER_TEXT"));
    assert_eq!(fs::read(&fixture.entry).unwrap(), before);
    assert_eq!(
        veac_ir::canonical_json(build_path(&fixture.entry).unwrap().envelope()).unwrap(),
        baseline
    );
}

#[test]
fn typed_text_template_edit_rebuilds_canonical_ir() {
    let fixture = fixture();
    let preview = apply_executable_source_edit_path(
        &fixture.entry,
        &batch(
            &fixture.entry,
            "item_for",
            TEMPLATE_BODY,
            "op_text_template",
        ),
    )
    .unwrap();
    let clip = preview.built.envelope().project.sequences[0]
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .find(|clip| clip.replaceable.is_some())
        .unwrap();
    assert_eq!(
        clip.replaceable.as_ref().unwrap().kind,
        veac_ir::SlotKind::Text
    );
    assert!(clip.template_editable_text);
    assert!(matches!(clip.source, ClipSource::Text { .. }));
}

#[test]
fn invalid_text_template_edit_is_atomic() {
    let fixture = fixture();
    let before = fs::read(&fixture.entry).unwrap();
    let invalid = TEMPLATE_BODY.replace("fill_fit_duration()", "fill_take_head()");
    let error = apply_executable_source_edit_path(
        &fixture.entry,
        &batch(
            &fixture.entry,
            "item_for",
            &invalid,
            "op_text_template_invalid",
        ),
    )
    .unwrap_err();
    assert!(matches!(error, SourceTransactionError::Program(_)));
    assert!(error.to_string().contains("TEMPLATE_SLOT_TEXT_FILL"));
    assert_eq!(fs::read(&fixture.entry).unwrap(), before);
}

struct Fixture {
    _directory: TempDir,
    entry: std::path::PathBuf,
}

fn fixture() -> Fixture {
    let directory = tempdir().unwrap();
    let entry = directory.path().join("main.veac");
    fs::create_dir(directory.path().join("assets")).unwrap();
    fs::write(&entry, SOURCE).unwrap();
    fs::write(directory.path().join("preview.veac"), PREVIEW).unwrap();
    fs::write(directory.path().join("assets/veac-example-zh.ttf"), FONT).unwrap();
    Fixture {
        _directory: directory,
        entry,
    }
}

fn batch(entry: &std::path::Path, target: &str, body: &str, operation: &str) -> SourceEditBatch {
    let prepared = prepare_path(entry).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new(operation).unwrap(),
        prepared.source_index().unwrap().revision().clone(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", target),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: body.to_owned(),
        },
    });
    batch
}
