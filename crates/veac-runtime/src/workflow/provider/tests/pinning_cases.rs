use std::ffi::OsString;

use veac_artifact::{ArtifactStore, ContentDigest};

use super::super::ProviderRunner;
use super::pinning_support::{rejecting_provider, valid_provider};
use super::support::fixture;

#[cfg(unix)]
#[test]
fn replacement_after_manifest_cannot_change_the_executed_provider() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = fixture();
    let original = valid_provider(&temp.path().join("provider-a.sh"), &fixture, "a");
    let replacement = rejecting_provider(&temp.path().join("provider-b.sh"), &fixture);
    let expected = ContentDigest::sha256(std::fs::read(&original).unwrap());
    let execution = ProviderRunner::new(&original)
        .with_arguments(arguments("replace", &original, &replacement))
        .run(
            &ArtifactStore::new(temp.path().join("store")),
            &fixture.request,
        )
        .unwrap();
    assert_eq!(execution.identity.executable, expected);
    assert_eq!(
        std::fs::read(&original).unwrap(),
        std::fs::read(&replacement).unwrap()
    );
    assert_eq!(execution.artifacts.len(), 1);
}

#[cfg(unix)]
#[test]
fn symlink_retarget_after_manifest_cannot_change_the_executed_provider() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let fixture = fixture();
    let original = valid_provider(&temp.path().join("provider-a.sh"), &fixture, "a");
    let replacement = rejecting_provider(&temp.path().join("provider-b.sh"), &fixture);
    let link = temp.path().join("provider-link.sh");
    symlink(&original, &link).unwrap();
    let expected = ContentDigest::sha256(std::fs::read(&original).unwrap());
    let execution = ProviderRunner::new(&link)
        .with_arguments(arguments("retarget", &link, &replacement))
        .run(
            &ArtifactStore::new(temp.path().join("store")),
            &fixture.request,
        )
        .unwrap();
    assert_eq!(execution.identity.executable, expected);
    assert_eq!(
        std::fs::canonicalize(&link).unwrap(),
        std::fs::canonicalize(&replacement).unwrap()
    );
    assert_eq!(execution.artifacts.len(), 1);
}

#[cfg(unix)]
#[test]
fn a_self_deleting_manifest_launch_does_not_remove_the_execution_launch() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = fixture();
    let provider = valid_provider(&temp.path().join("provider.sh"), &fixture, "self-delete");
    let execution = ProviderRunner::new(&provider)
        .with_arguments(["self-delete"])
        .run(
            &ArtifactStore::new(temp.path().join("store")),
            &fixture.request,
        )
        .unwrap();
    assert!(provider.exists());
    assert_eq!(execution.artifacts.len(), 1);
}

#[cfg(unix)]
#[test]
fn executable_bytes_bind_execution_digests_and_artifact_keys() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = fixture();
    let first = valid_provider(&temp.path().join("provider-a.sh"), &fixture, "a");
    let second = valid_provider(&temp.path().join("provider-b.sh"), &fixture, "b");
    let first_execution = ProviderRunner::new(first)
        .run(
            &ArtifactStore::new(temp.path().join("store-a")),
            &fixture.request,
        )
        .unwrap();
    let second_execution = ProviderRunner::new(second)
        .run(
            &ArtifactStore::new(temp.path().join("store-b")),
            &fixture.request,
        )
        .unwrap();
    assert_eq!(
        first_execution.identity.provider,
        second_execution.identity.provider
    );
    assert_ne!(
        first_execution.identity.executable,
        second_execution.identity.executable
    );
    assert_ne!(
        first_execution.identity.digest,
        second_execution.identity.digest
    );
    assert_ne!(
        first_execution.artifacts[0].key,
        second_execution.artifacts[0].key
    );
}

#[cfg(unix)]
#[test]
fn one_runner_reuses_a_stable_master_identity_across_executions() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = fixture();
    let provider = valid_provider(&temp.path().join("provider.sh"), &fixture, "stable");
    let runner = ProviderRunner::new(provider);
    let store = ArtifactStore::new(temp.path().join("store"));
    let first = runner.run(&store, &fixture.request).unwrap();
    let second = runner.run(&store, &fixture.request).unwrap();
    assert_eq!(first.identity, second.identity);
    assert_eq!(first.response, second.response);
    assert_eq!(first.artifacts, second.artifacts);
}

fn arguments(
    action: &str,
    target: &std::path::Path,
    replacement: &std::path::Path,
) -> Vec<OsString> {
    vec![action.into(), target.into(), replacement.into()]
}
