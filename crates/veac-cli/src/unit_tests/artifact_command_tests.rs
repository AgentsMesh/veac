use tempfile::tempdir;

use super::support::{
    add_second_video, canonical_project, FakeEnvironment, GENERATED_SOURCE, MEDIA_SOURCE,
};

#[test]
fn manifest_supports_stdout_and_propagates_failures() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    crate::commands::manifest(
        &project,
        None,
        None,
        None,
        None,
        &FakeEnvironment::success(),
    )
    .unwrap();
    let failed = FakeEnvironment {
        fail_version: true,
        ..FakeEnvironment::success()
    };
    let failed_output = temp.path().join("failed-manifest.json");
    assert!(
        crate::commands::manifest(&project, None, None, None, Some(&failed_output), &failed)
            .unwrap_err()
            .to_string()
            .contains("FAKE_VERSION")
    );
    assert!(!failed_output.exists());
    assert!(crate::commands::manifest(
        &project,
        None,
        None,
        None,
        Some(&project),
        &FakeEnvironment::success(),
    )
    .unwrap_err()
    .to_string()
    .contains("OUTPUT_OVERWRITES_INPUT"));
}

#[test]
fn manifest_cannot_overwrite_a_reachable_material() {
    let temp = tempdir().unwrap();
    let material = temp.path().join("clip.mp4");
    std::fs::write(&material, "fixture").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let error = crate::commands::manifest(
        &project,
        None,
        None,
        None,
        Some(&material),
        &FakeEnvironment::success(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("OUTPUT_OVERWRITES_INPUT"));
}

#[test]
fn package_rejects_a_file_destination_before_artifact_io() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let destination = temp.path().join("file");
    std::fs::write(&destination, "not a directory").unwrap();
    assert!(crate::commands::package(
        &project,
        None,
        None,
        None,
        &destination,
        &FakeEnvironment::success()
    )
    .unwrap_err()
    .to_string()
    .contains("OUTPUT_IS_FILE"));
}

#[test]
fn package_maps_artifact_publication_failures() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let destination = temp.path().join("package");
    std::fs::create_dir_all(destination.join("project.veac.json")).unwrap();
    let error = crate::commands::package(
        &project,
        None,
        None,
        None,
        &destination,
        &FakeEnvironment::success(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("PACKAGE_FAILED"));
}

#[test]
fn manifest_contains_every_authored_deliverable() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    add_second_video(&project);
    let output = temp.path().join("manifest.json");
    crate::commands::manifest(
        &project,
        None,
        None,
        None,
        Some(&output),
        &FakeEnvironment::success(),
    )
    .unwrap();
    let manifest: veac_artifact::BuildManifest =
        serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
    let names: Vec<_> = manifest
        .output
        .deliverables
        .iter()
        .map(|value| value.target.file_name().unwrap())
        .collect();
    assert_eq!(names, ["render.mp4", "second.mp4"]);
}

#[test]
fn manifest_uses_the_complete_environment_ffmpeg_fingerprint() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let first = temp.path().join("first.json");
    let second = temp.path().join("second.json");
    let first_environment = FakeEnvironment::success();
    let mut second_environment = FakeEnvironment::success();
    second_environment.fingerprint_configuration =
        veac_artifact::ContentDigest::sha256("different configuration");

    crate::commands::manifest(&project, None, None, None, Some(&first), &first_environment)
        .unwrap();
    crate::commands::manifest(
        &project,
        None,
        None,
        None,
        Some(&second),
        &second_environment,
    )
    .unwrap();
    let first: veac_artifact::BuildManifest =
        serde_json::from_slice(&std::fs::read(first).unwrap()).unwrap();
    let second: veac_artifact::BuildManifest =
        serde_json::from_slice(&std::fs::read(second).unwrap()).unwrap();

    assert_eq!(first.tools[0].version, second.tools[0].version);
    assert_eq!(
        first.tools[0].configuration,
        first_environment.fingerprint_configuration
    );
    assert_eq!(
        second.tools[0].configuration,
        second_environment.fingerprint_configuration
    );
    assert_ne!(
        veac_artifact::build_manifest_hash(&first).unwrap(),
        veac_artifact::build_manifest_hash(&second).unwrap()
    );
}
