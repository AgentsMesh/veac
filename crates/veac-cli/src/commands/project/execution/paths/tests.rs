use super::{existing, inside, writable, ProjectExecutionRoots};

#[test]
fn root_confinement_accepts_descendants_and_rejects_escapes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    std::fs::create_dir(&root).unwrap();

    let child = root.join("child");
    assert_eq!(inside(&root, child.clone(), "child").unwrap(), child);
    let error = inside(&root, temp.path().join("outside"), "outside").unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "PROJECT_PATH_ESCAPE");
}

#[test]
fn existing_and_writable_paths_report_io_failures() {
    let temp = tempfile::tempdir().unwrap();
    let root = std::fs::canonicalize(temp.path()).unwrap();
    let present = root.join("present");
    std::fs::create_dir(&present).unwrap();
    assert_eq!(existing(&root, "present", "present").unwrap(), present);
    assert!(existing(&root, "missing", "missing").is_err());

    let occupied = root.join("occupied");
    std::fs::write(&occupied, b"file").unwrap();
    let error = writable(&root, "occupied/child", "child").unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "PROJECT_PATH_CREATE");
    let error = writable(&root, "occupied", "occupied").unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "PROJECT_PATH_CREATE");

    let nested = writable(&root, "new/nested", "nested").unwrap();
    assert_eq!(nested, root.join("new/nested"));
    assert!(nested.is_dir());
}

#[test]
fn writable_rejects_non_relative_paths_before_creation() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    let outside = temp.path().join("outside");
    std::fs::create_dir(&root).unwrap();

    for relative in [outside.to_str().unwrap(), "../outside"] {
        let error = writable(&root, relative, "build").unwrap_err();
        assert_eq!(error.diagnostics()[0].code, "PROJECT_PATH_ESCAPE");
    }
    assert!(!outside.exists());
}

#[cfg(unix)]
#[test]
fn writable_rejects_symlinked_ancestors_without_target_side_effects() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    let outside = temp.path().join("outside");
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(&outside).unwrap();
    symlink(&outside, root.join("escape")).unwrap();

    let error = writable(&root, "escape/created/deep", "build").unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "PROJECT_PATH_SYMLINK");
    assert!(!outside.join("created").exists());
}

#[cfg(unix)]
#[test]
fn writable_rejects_internal_symlink_aliases_without_target_side_effects() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    let target = root.join("target");
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(&target).unwrap();
    symlink(&target, root.join("alias")).unwrap();

    let error = writable(&root, "alias/created", "cache").unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "PROJECT_PATH_SYMLINK");
    assert!(!target.join("created").exists());
}

#[test]
fn canonical_root_authorities_reject_aliases_and_nesting() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let material = temp.path().join("material");
    let build = temp.path().join("build");
    let cache = temp.path().join("cache");
    let delivery = temp.path().join("delivery");
    for path in [&source, &material, &build, &cache, &delivery] {
        std::fs::create_dir(path).unwrap();
    }
    let mut roots = ProjectExecutionRoots {
        source: source.clone(),
        material,
        build,
        cache,
        delivery,
    };
    assert!(roots.validate_authorities().is_ok());

    roots.delivery = source.join("nested");
    let error = roots.validate_authorities().unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "PROJECT_PATH_AUTHORITY");
    roots.delivery = source;
    assert!(roots.validate_authorities().is_err());
}

#[test]
fn execution_roots_reject_a_package_mount_inside_any_authority() {
    let package_root = std::fs::canonicalize(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib/veac-components"),
    )
    .unwrap();
    let packages = veac_build::ProjectPackageSet::capture(&[package_root.clone()]).unwrap();
    let roots = ProjectExecutionRoots {
        source: package_root,
        material: std::path::PathBuf::from("/tmp/material"),
        build: std::path::PathBuf::from("/tmp/build"),
        cache: std::path::PathBuf::from("/tmp/cache"),
        delivery: std::path::PathBuf::from("/tmp/delivery"),
    };
    let error = roots.validate_package_roots(&packages).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "PROJECT_PACKAGE_AUTHORITY");
}
