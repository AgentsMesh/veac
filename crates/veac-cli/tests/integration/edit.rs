use std::path::{Path, PathBuf};

use veac_ir::{EditBatch, EditOperation, OperationId};

use super::support::*;

#[test]
fn cli_edit_applies_in_place_and_emits_canonical_outcome() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, GENERATED_SOURCE);
    let batch = edit_batch(&temp, &project, 0, "op_e2e_apply");
    let output = veac()
        .args(["edit", path(&project), path(&batch)])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let outcome: veac_ir::EditOutcome = serde_json::from_slice(&output.stdout).unwrap();
    assert!(matches!(
        outcome,
        veac_ir::EditOutcome::Applied {
            new_revision: 1,
            ..
        }
    ));
    let edited = read_project(&project);
    assert_eq!(edited.project.revision, 1);
    assert!(!edited.project.sequences[0].tracks[0].clips[0].enabled);
}

#[test]
fn cli_edit_dry_run_and_explicit_output_preserve_the_source() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, GENERATED_SOURCE);
    let original = std::fs::read_to_string(&project).unwrap();
    let batch = edit_batch(&temp, &project, 0, "op_e2e_preview");
    let destination = temp.path().join("edited.json");
    veac()
        .args([
            "edit",
            path(&project),
            path(&batch),
            "--output",
            path(&destination),
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\":\"applied\""));
    assert!(!destination.exists());
    assert_eq!(std::fs::read_to_string(&project).unwrap(), original);

    veac()
        .args([
            "edit",
            path(&project),
            path(&batch),
            "-o",
            path(&destination),
        ])
        .assert()
        .success();
    assert_eq!(read_project(&destination).project.revision, 1);
    assert_eq!(std::fs::read_to_string(&project).unwrap(), original);
}

#[test]
fn cli_edit_conflicts_are_machine_readable_and_non_mutating() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, GENERATED_SOURCE);
    let batch = edit_batch(&temp, &project, 7, "op_e2e_stale");
    let original = std::fs::read_to_string(&project).unwrap();
    let output = veac()
        .args(["edit", path(&project), path(&batch)])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let outcome: veac_ir::EditOutcome = serde_json::from_slice(&output.stdout).unwrap();
    assert!(matches!(
        outcome,
        veac_ir::EditOutcome::Conflict {
            current_revision: 0,
            ..
        }
    ));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error[STALE_REVISION]"));
    assert!(stderr.contains("/base_revision"));
    assert!(stderr.contains("object: op_e2e_stale"));
    assert_eq!(std::fs::read_to_string(&project).unwrap(), original);
}

#[test]
fn cli_edit_protects_batch_and_local_material_paths() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, MEDIA_SOURCE);
    let batch = edit_batch(&temp, &project, 0, "op_e2e_guard");
    veac()
        .args([
            "edit",
            path(&project),
            path(&batch),
            "--output",
            path(&batch),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("OUTPUT_OVERWRITES_INPUT"));
    veac()
        .args([
            "edit",
            path(&project),
            path(&batch),
            "--output",
            path(&temp.path().join("clip.mp4")),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("OUTPUT_OVERWRITES_INPUT"));
}

fn edit_batch(temp: &TempDir, project: &Path, revision: u64, id: &str) -> PathBuf {
    let clip_id = read_project(project).project.sequences[0].tracks[0].clips[0]
        .id
        .clone();
    let batch = EditBatch {
        operation_id: OperationId::new(id).unwrap(),
        base_revision: revision,
        atomic: true,
        preconditions: vec![],
        operations: vec![EditOperation::SetClipEnabled {
            clip_id,
            enabled: false,
        }],
    };
    let file = temp.path().join(format!("{id}.json"));
    std::fs::write(&file, veac_ir::canonical_edit_batch_json(&batch).unwrap()).unwrap();
    file
}

fn read_project(path: &Path) -> veac_ir::ProjectEnvelope {
    veac_ir::decode_canonical_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn path(path: &Path) -> &str {
    path.to_str().unwrap()
}
