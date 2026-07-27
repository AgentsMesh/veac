use std::path::{Path, PathBuf};

use tempfile::TempDir;
use veac_ir::{EditBatch, EditOperation, MaterialSource, OperationId, StructureEdit};

use super::support::{canonical_project, MEDIA_SOURCE};

#[test]
fn set_material_cannot_make_edit_output_overwrite_its_new_local_source() {
    let temp = tempfile::tempdir().unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let envelope = crate::canonical::load(&project).unwrap();
    let current = &envelope.project.materials[0];
    let target = source(&temp, "set-target.mp4");
    let mut material = current.clone();
    material.source = file_source(&target);
    let edit = StructureEdit::SetMaterial {
        material_id: current.id.clone(),
        material: Box::new(material),
    };

    assert_guarded(&temp, &project, target, edit, "op_set_material_guard");
}

#[test]
fn relink_material_cannot_make_edit_output_overwrite_its_new_local_source() {
    let temp = tempfile::tempdir().unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let envelope = crate::canonical::load(&project).unwrap();
    let current = &envelope.project.materials[0];
    let target = source(&temp, "relink-target.mp4");
    let edit = StructureEdit::RelinkMaterial {
        material_id: current.id.clone(),
        source: file_source(&target),
        identity: current.identity.clone(),
        probe: current.probe.clone().map(Box::new),
    };

    assert_guarded(&temp, &project, target, edit, "op_relink_material_guard");
}

fn assert_guarded(temp: &TempDir, project: &Path, target: PathBuf, edit: StructureEdit, id: &str) {
    let batch = EditBatch {
        operation_id: OperationId::new(id).unwrap(),
        base_revision: 0,
        atomic: true,
        preconditions: vec![],
        operations: vec![EditOperation::EditStructure { edit }],
    };
    let batch_path = temp.path().join(format!("{id}.json"));
    std::fs::write(
        &batch_path,
        veac_ir::canonical_edit_batch_json(&batch).unwrap(),
    )
    .unwrap();
    let error = crate::commands::edit(project, &batch_path, Some(&target), false).unwrap_err();
    assert!(error.to_string().contains("OUTPUT_OVERWRITES_INPUT"));
    assert_eq!(std::fs::read(target).unwrap(), b"source");
}

fn source(temp: &TempDir, name: &str) -> PathBuf {
    let path = temp.path().join(name);
    std::fs::write(&path, b"source").unwrap();
    path
}

fn file_source(path: &Path) -> MaterialSource {
    MaterialSource::File {
        uri: path.file_name().unwrap().to_string_lossy().into_owned(),
    }
}
