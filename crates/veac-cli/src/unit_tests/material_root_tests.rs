use tempfile::tempdir;

use super::support::{canonical_project, GENERATED_SOURCE, MEDIA_SOURCE};
use crate::material_root::MaterialRoot;

#[test]
fn roots_default_to_their_declared_authority_and_explicit_roots_are_marked() {
    let temp = tempdir().unwrap();
    let source_root = directory(temp.path(), "source");
    let project_root = directory(temp.path(), "project");
    let explicit = directory(temp.path(), "materials");
    let project = project_root.join("project.json");

    let source = MaterialRoot::for_build(&source_root, None).unwrap();
    let canonical_source = source_root.canonicalize().unwrap();
    assert_eq!(source.path(), canonical_source);
    assert!(!source.is_explicit());

    let project = MaterialRoot::for_project(&project, None).unwrap();
    assert_eq!(project.path(), project_root.canonicalize().unwrap());
    assert!(!project.is_explicit());

    let current = MaterialRoot::for_project(std::path::Path::new("project.json"), None).unwrap();
    assert_eq!(
        current.path(),
        std::env::current_dir().unwrap().canonicalize().unwrap()
    );

    let explicit_root =
        MaterialRoot::for_project(&project_root.join("project.json"), Some(&explicit)).unwrap();
    assert_eq!(explicit_root.path(), explicit.canonicalize().unwrap());
    assert!(explicit_root.is_explicit());
}

#[test]
fn roots_require_an_existing_directory() {
    let temp = tempdir().unwrap();
    let file = temp.path().join("file");
    std::fs::write(&file, b"file").unwrap();

    let missing = MaterialRoot::for_build(&temp.path().join("missing"), None).unwrap_err();
    assert!(missing.to_string().contains("PATH_UNAVAILABLE"));
    let not_directory = MaterialRoot::for_build(&file, None).unwrap_err();
    assert!(not_directory.to_string().contains("NOT_A_DIRECTORY"));
}

#[test]
fn nested_materials_resolve_to_canonical_paths() {
    let temp = tempdir().unwrap();
    let assets = directory(temp.path(), "assets");
    let material = assets.join("clip.bin");
    std::fs::write(&material, b"material").unwrap();
    let root = MaterialRoot::for_build(temp.path(), None).unwrap();

    assert_eq!(
        root.resolve_file("assets/clip.bin").unwrap(),
        material.canonicalize().unwrap()
    );
}

#[test]
fn local_path_inventory_uses_the_root_and_ignores_remote_materials() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    let root = MaterialRoot::for_project(&project, None).unwrap();
    assert_eq!(
        root.local_paths(&envelope),
        vec![temp.path().canonicalize().unwrap().join("clip.mp4")]
    );

    envelope.project.materials[0].source = veac_ir::MaterialSource::Remote {
        uri: "https://example.test/clip.mp4".to_owned(),
    };
    assert!(root.local_paths(&envelope).is_empty());
}

#[cfg(unix)]
#[test]
fn root_aliases_are_allowed_but_material_symlink_escapes_are_rejected() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let root = directory(temp.path(), "root");
    let outside = directory(temp.path(), "outside");
    let outside_file = outside.join("clip.bin");
    std::fs::write(&outside_file, b"outside").unwrap();
    let root_alias = temp.path().join("root-alias");
    symlink(&root, &root_alias).unwrap();

    let materials = MaterialRoot::for_build(&root_alias, None).unwrap();
    assert_eq!(materials.path(), root.canonicalize().unwrap());
    let contained = root.join("contained.bin");
    std::fs::write(&contained, b"contained").unwrap();
    symlink(&contained, root.join("contained-alias.bin")).unwrap();
    assert_eq!(
        materials.resolve_file("contained-alias.bin").unwrap(),
        contained.canonicalize().unwrap()
    );
    symlink(&outside_file, root.join("leaf.bin")).unwrap();
    assert!(materials
        .resolve_file("leaf.bin")
        .unwrap_err()
        .to_string()
        .contains("MATERIAL_OUTSIDE_ROOT"));

    symlink(&outside, root.join("nested")).unwrap();
    assert!(materials
        .resolve_file("nested/clip.bin")
        .unwrap_err()
        .to_string()
        .contains("MATERIAL_OUTSIDE_ROOT"));
}

#[test]
fn detached_ir_requires_an_explicit_material_root_only_for_local_materials() {
    let temp = tempdir().unwrap();
    let detached = directory(temp.path(), "detached").join("project.json");
    let media_project = canonical_project(&temp, MEDIA_SOURCE);
    let media = crate::canonical::load(&media_project).unwrap();
    let default_root = MaterialRoot::for_build(temp.path(), None).unwrap();
    crate::material_root::require_detached_output_contract(
        &media,
        temp.path(),
        &default_root,
        &temp.path().join("project.json"),
    )
    .unwrap();
    let error = crate::material_root::require_detached_output_contract(
        &media,
        temp.path(),
        &default_root,
        &detached,
    )
    .unwrap_err();
    assert!(error.to_string().contains("OUTPUT_MATERIAL_BASE_MISMATCH"));

    let explicit_root = MaterialRoot::for_build(temp.path(), Some(temp.path())).unwrap();
    crate::material_root::require_detached_output_contract(
        &media,
        temp.path(),
        &explicit_root,
        &detached,
    )
    .unwrap();

    let generated_project = canonical_project(&temp, GENERATED_SOURCE);
    let generated = crate::canonical::load(&generated_project).unwrap();
    crate::material_root::require_detached_output_contract(
        &generated,
        temp.path(),
        &default_root,
        &detached,
    )
    .unwrap();
}

fn directory(root: &std::path::Path, name: &str) -> std::path::PathBuf {
    let path = root.join(name);
    std::fs::create_dir(&path).unwrap();
    path
}
