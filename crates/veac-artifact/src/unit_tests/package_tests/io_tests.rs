use std::fs;

use crate::{
    package::io::{copy_verified, write_manifest, write_project},
    test_support::media_identity,
    ArtifactErrorKind, ContentDigest,
};

#[test]
fn verified_copy_cleans_failed_temporary_files() {
    let identity = media_identity(b"expected");
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source.bin");
    let package = root.path().join("output");
    let destination = package.join("payload.bin");
    fs::write(&source, b"different").unwrap();
    let error = copy_verified(&source, &package, "payload.bin", &identity).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::IdentityMismatch);
    assert!(!destination.exists());
    assert_eq!(
        fs::read_dir(destination.parent().unwrap()).unwrap().count(),
        0
    );
}

#[cfg(unix)]
#[test]
fn verified_copy_ignores_predictable_temporary_symlink_without_touching_victim() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    let package = root.path().join("package");
    fs::create_dir(&package).unwrap();
    let source = root.path().join("source.bin");
    let victim = root.path().join("victim.bin");
    fs::write(&source, b"expected").unwrap();
    fs::write(&victim, b"protected").unwrap();
    let destination = package.join("payload.bin");
    let temporary = package.join(format!("payload.bin.{}.tmp", std::process::id()));
    symlink(&victim, &temporary).unwrap();

    copy_verified(
        &source,
        &package,
        "payload.bin",
        &media_identity(b"expected"),
    )
    .unwrap();
    assert_eq!(fs::read(victim).unwrap(), b"protected");
    assert_eq!(fs::read(destination).unwrap(), b"expected");
    assert!(fs::symlink_metadata(temporary)
        .unwrap()
        .file_type()
        .is_symlink());
}

#[test]
fn project_and_manifest_ignore_predictable_existing_temporary_files() {
    let root = tempfile::tempdir().unwrap();
    let package = root.path().join("package");
    fs::create_dir(&package).unwrap();
    let suffix = format!(".{}.tmp", std::process::id());
    let project_temp = package.join(format!("project.veac.json{suffix}"));
    let manifest_temp = package.join(format!("package.json{suffix}"));
    fs::write(&project_temp, b"project sentinel").unwrap();
    fs::write(&manifest_temp, b"manifest sentinel").unwrap();

    write_project(&package, b"project", &ContentDigest::sha256(b"project")).unwrap();
    write_manifest(&package, b"manifest").unwrap();

    assert_eq!(fs::read(project_temp).unwrap(), b"project sentinel");
    assert_eq!(fs::read(manifest_temp).unwrap(), b"manifest sentinel");
    assert_eq!(
        fs::read(package.join("project.veac.json")).unwrap(),
        b"project"
    );
    assert_eq!(fs::read(package.join("package.json")).unwrap(), b"manifest");
}

#[cfg(unix)]
#[test]
fn verified_copy_rejects_existing_symlink_and_directory_outputs() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    let package = root.path().join("package");
    fs::create_dir(&package).unwrap();
    let source = root.path().join("source.bin");
    let victim = root.path().join("victim.bin");
    fs::write(&source, b"expected").unwrap();
    fs::write(&victim, b"protected").unwrap();
    let output = package.join("payload.bin");
    symlink(&victim, &output).unwrap();

    let identity = media_identity(b"expected");
    let error = copy_verified(&source, &package, "payload.bin", &identity).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
    assert_eq!(fs::read(&victim).unwrap(), b"protected");

    fs::remove_file(&output).unwrap();
    fs::create_dir(&output).unwrap();
    let error = copy_verified(&source, &package, "payload.bin", &identity).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
}
