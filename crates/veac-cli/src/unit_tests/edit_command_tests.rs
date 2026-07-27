use std::path::{Path, PathBuf};

use tempfile::{tempdir, TempDir};
use veac_ir::{EditBatch, EditOperation, ItemId, OperationId};

use super::support::{canonical_project, GENERATED_SOURCE, MEDIA_SOURCE};

#[test]
fn edit_applies_in_place_and_retries_idempotently() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let batch = write_batch(&temp, 0, "op_cli_apply", first_clip(&project), false);
    crate::commands::edit(&project, &batch, None, false).unwrap();
    let edited = crate::canonical::load(&project).unwrap();
    assert_eq!(edited.project.revision, 1);
    assert!(!edited.project.sequences[0].tracks[0].clips[0].enabled);

    crate::commands::edit(&project, &batch, None, false).unwrap();
    assert_eq!(
        crate::canonical::load(&project).unwrap().project.revision,
        1
    );
}

#[test]
fn edit_supports_dry_run_and_explicit_output() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let original = std::fs::read_to_string(&project).unwrap();
    let batch = write_batch(&temp, 0, "op_cli_preview", first_clip(&project), false);
    let preview = temp.path().join("preview.json");
    crate::commands::edit(&project, &batch, Some(&preview), true).unwrap();
    assert_eq!(std::fs::read_to_string(&project).unwrap(), original);
    assert!(!preview.exists());

    crate::commands::edit(&project, &batch, Some(&preview), false).unwrap();
    assert_eq!(
        crate::canonical::load(&project).unwrap().project.revision,
        0
    );
    assert_eq!(
        crate::canonical::load(&preview).unwrap().project.revision,
        1
    );
}

#[test]
fn edit_conflicts_and_rejections_never_write() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let original = std::fs::read_to_string(&project).unwrap();
    let stale = write_batch(&temp, 9, "op_cli_stale", first_clip(&project), false);
    let error = crate::commands::edit(&project, &stale, None, false).unwrap_err();
    assert!(error.to_string().contains("STALE_REVISION"));
    assert!(error.to_string().contains("/base_revision"));

    let rejected = write_batch(
        &temp,
        0,
        "op_cli_rejected",
        ItemId::new("itm_absent").unwrap(),
        false,
    );
    assert!(crate::commands::edit(&project, &rejected, None, false)
        .unwrap_err()
        .to_string()
        .contains("EDIT_REJECTED"));
    assert_eq!(std::fs::read_to_string(&project).unwrap(), original);
}

#[test]
fn edit_rejects_malformed_batches_and_protected_outputs() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let batch = write_batch(&temp, 0, "op_cli_guard", first_clip(&project), false);
    assert!(crate::commands::edit(&project, &batch, Some(&batch), false)
        .unwrap_err()
        .to_string()
        .contains("OUTPUT_OVERWRITES_INPUT"));
    assert!(
        crate::commands::edit(&project, &batch, Some(&temp.path().join("clip.mp4")), false,)
            .unwrap_err()
            .to_string()
            .contains("OUTPUT_OVERWRITES_INPUT")
    );

    let malformed = temp.path().join("malformed.json");
    std::fs::write(&malformed, r#"{"atomic":true}"#).unwrap();
    assert!(crate::commands::edit(&project, &malformed, None, false)
        .unwrap_err()
        .to_string()
        .contains("EDIT_BATCH_JSON"));
}

fn first_clip(project: &Path) -> ItemId {
    crate::canonical::load(project).unwrap().project.sequences[0].tracks[0].clips[0]
        .id
        .clone()
}

fn write_batch(temp: &TempDir, revision: u64, id: &str, clip_id: ItemId, enabled: bool) -> PathBuf {
    let batch = EditBatch {
        operation_id: OperationId::new(id).unwrap(),
        base_revision: revision,
        atomic: true,
        preconditions: vec![],
        operations: vec![EditOperation::SetClipEnabled { clip_id, enabled }],
    };
    let path = temp.path().join(format!("{id}.json"));
    std::fs::write(&path, veac_ir::canonical_edit_batch_json(&batch).unwrap()).unwrap();
    path
}
