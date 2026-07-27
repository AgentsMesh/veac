use tempfile::tempdir;

use super::support::{canonical_project, FakeEnvironment, GENERATED_SOURCE, MEDIA_SOURCE};

#[test]
fn render_output_defaults_and_explicit_paths_are_bound() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let mut prepared =
        crate::planning::prepare(&project, None, &FakeEnvironment::success()).unwrap();
    let plan_hash = veac_plan::plan_hash(&prepared.plan).unwrap();
    let default = crate::output::bind_render_outputs(&mut prepared, None).unwrap();
    let base = std::fs::canonicalize(temp.path()).unwrap();
    assert_eq!(default, vec![base.join("render.mp4")]);
    assert_eq!(prepared.bindings.outputs().len(), 1);

    let explicit = temp.path().join("chosen");
    std::fs::create_dir(&explicit).unwrap();
    let explicit = std::fs::canonicalize(explicit).unwrap();
    let bound = crate::output::bind_render_outputs(&mut prepared, Some(&explicit)).unwrap();
    assert_eq!(bound, vec![explicit.join("render.mp4")]);
    assert_eq!(veac_plan::plan_hash(&prepared.plan).unwrap(), plan_hash);
}

#[test]
fn outputs_cannot_alias_projects_or_materials() {
    let temp = tempdir().unwrap();
    let source = canonical_project(&temp, GENERATED_SOURCE);
    let project = temp.path().join("render.mp4");
    std::fs::rename(&source, &project).unwrap();
    let mut prepared =
        crate::planning::prepare(&project, None, &FakeEnvironment::success()).unwrap();
    assert!(crate::output::bind_render_outputs(&mut prepared, None)
        .unwrap_err()
        .to_string()
        .contains("OUTPUT_OVERWRITES_INPUT"));

    std::fs::write(temp.path().join("clip.mp4"), "fixture").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    envelope.project.render_configs[0].deliverables[0].file_name = "clip.mp4".to_owned();
    std::fs::write(&project, veac_ir::canonical_json(&envelope).unwrap()).unwrap();
    let mut prepared =
        crate::planning::prepare(&project, None, &FakeEnvironment::success()).unwrap();
    assert!(crate::output::bind_render_outputs(&mut prepared, None)
        .unwrap_err()
        .to_string()
        .contains("OUTPUT_OVERWRITES_INPUT"));
}

#[test]
fn destination_rejects_directories_symlinks_and_missing_parents() {
    let temp = tempdir().unwrap();
    let protected_path = temp.path().join("protected");
    std::fs::write(&protected_path, "protected").unwrap();
    let protected = std::fs::canonicalize(protected_path).unwrap();
    assert!(
        crate::output::guarded_write_many(temp.path(), std::iter::once(protected.as_path()))
            .unwrap_err()
            .to_string()
            .contains("OUTPUT_IS_DIRECTORY")
    );
    assert!(crate::output::guarded_write_many(
        &temp.path().join("missing/out.json"),
        std::iter::once(protected.as_path())
    )
    .unwrap_err()
    .to_string()
    .contains("OUTPUT_PARENT_UNAVAILABLE"));
    assert!(crate::output::guarded_write_many(
        std::path::Path::new("/"),
        std::iter::once(protected.as_path())
    )
    .unwrap_err()
    .to_string()
    .contains("INVALID_OUTPUT_PATH"));

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&protected, temp.path().join("link.json")).unwrap();
        assert!(crate::output::guarded_write_many(
            &temp.path().join("link.json"),
            std::iter::once(protected.as_path())
        )
        .unwrap_err()
        .to_string()
        .contains("OUTPUT_SYMLINK"));
    }
}

#[test]
fn protected_input_normalization_preserves_unresolvable_paths() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("output.json");
    let missing = temp.path().join("missing/input.json");
    let protected = [missing.as_path(), std::path::Path::new("")];
    assert_eq!(
        crate::output::guarded_write_many(&output, protected.into_iter()).unwrap(),
        std::fs::canonicalize(temp.path())
            .unwrap()
            .join("output.json")
    );
}

#[test]
fn render_destination_must_be_an_existing_directory() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let mut prepared =
        crate::planning::prepare(&project, None, &FakeEnvironment::success()).unwrap();
    let error = crate::output::bind_render_outputs(
        &mut prepared,
        Some(&temp.path().join("missing-directory")),
    )
    .unwrap_err();
    assert!(error.to_string().contains("OUTPUT_DIRECTORY_REQUIRED"));
}

#[test]
fn local_material_path_collection_ignores_remote_locators() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    assert_eq!(
        crate::canonical::local_material_paths(&envelope, &project),
        vec![project.parent().unwrap().join("clip.mp4")]
    );
    envelope.project.materials[0].source = veac_ir::MaterialSource::Remote {
        uri: "https://example.test/clip.mp4".to_owned(),
    };
    assert!(crate::canonical::local_material_paths(&envelope, &project).is_empty());
}

#[test]
fn atomic_io_replaces_files_and_cleans_failed_temps() {
    let temp = tempdir().unwrap();
    assert!(
        crate::fs::atomic_write(std::path::Path::new("/"), "invalid")
            .unwrap_err()
            .to_string()
            .contains("INVALID_OUTPUT_PATH")
    );
    let file = temp.path().join("value.txt");
    std::fs::write(&file, "old").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    crate::fs::atomic_write(&file, "new").unwrap();
    assert_eq!(crate::fs::read_utf8(&file, "value").unwrap(), "new");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&file).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    assert!(
        crate::fs::atomic_write(temp.path(), "cannot replace a directory")
            .unwrap_err()
            .to_string()
            .contains("WRITE_FAILED")
    );
    assert!(!std::fs::read_dir(temp.path()).unwrap().any(|entry| entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with(".veac-stage-")));
}
