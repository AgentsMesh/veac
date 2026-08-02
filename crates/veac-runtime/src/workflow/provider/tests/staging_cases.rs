use std::fs;
use std::time::{Duration, Instant};

use veac_provider::{ProviderOutput, SeparatedStem, StemKind, VocalSeparationResult};

use super::super::{staging, ProviderResourceLimits, WorkflowErrorKind};
use super::support::fixture;

#[test]
fn staging_accepts_the_exact_declared_payload_set_and_rejects_duplicate_keys() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = fixture();
    fs::write(temp.path().join(&fixture.payload_name), b"speech").unwrap();
    let payloads = verify(
        temp.path(),
        &fixture.response,
        ProviderResourceLimits::default(),
    )
    .unwrap();
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0].path, temp.path().join(&fixture.payload_name));

    let artifact = fixture.response.output.artifacts()[0].clone();
    let mut duplicate = fixture.response.clone();
    duplicate.output = ProviderOutput::VocalSeparation(VocalSeparationResult {
        stems: vec![
            SeparatedStem {
                kind: StemKind::Vocals,
                label: "first".into(),
                audio: artifact.clone(),
                leakage: 0.0,
            },
            SeparatedStem {
                kind: StemKind::Music,
                label: "second".into(),
                audio: artifact,
                leakage: 0.0,
            },
        ],
    });
    assert_eq!(
        verify(temp.path(), &duplicate, ProviderResourceLimits::default(),)
            .err()
            .unwrap()
            .kind,
        WorkflowErrorKind::UnsafeStaging
    );
}

#[test]
fn staging_deducts_declared_per_artifact_total_and_count_budgets_before_hashing() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = fixture();
    fs::write(temp.path().join(&fixture.payload_name), b"speech").unwrap();

    let limits = ProviderResourceLimits {
        max_payload_bytes: 5,
        ..ProviderResourceLimits::default()
    };
    assert_eq!(
        verify(temp.path(), &fixture.response, limits)
            .unwrap_err()
            .kind,
        WorkflowErrorKind::ResourceLimit
    );
    let limits = ProviderResourceLimits {
        max_total_payload_bytes: 5,
        ..ProviderResourceLimits::default()
    };
    assert_eq!(
        verify(temp.path(), &fixture.response, limits)
            .unwrap_err()
            .kind,
        WorkflowErrorKind::ResourceLimit
    );

    let artifact = fixture.response.output.artifacts()[0].clone();
    let mut two = fixture.response.clone();
    two.output = ProviderOutput::VocalSeparation(VocalSeparationResult {
        stems: vec![
            SeparatedStem {
                kind: StemKind::Vocals,
                label: "first".into(),
                audio: artifact.clone(),
                leakage: 0.0,
            },
            SeparatedStem {
                kind: StemKind::Music,
                label: "second".into(),
                audio: artifact,
                leakage: 0.0,
            },
        ],
    });
    let limits = ProviderResourceLimits {
        max_artifacts: 1,
        ..ProviderResourceLimits::default()
    };
    assert_eq!(
        verify(temp.path(), &two, limits).unwrap_err().kind,
        WorkflowErrorKind::ResourceLimit
    );
}

#[cfg(unix)]
#[test]
fn staging_rejects_file_and_symlink_roots() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let fixture = fixture();
    let file = temp.path().join("not-a-directory");
    fs::write(&file, b"x").unwrap();
    assert_eq!(
        verify(&file, &fixture.response, ProviderResourceLimits::default(),)
            .err()
            .unwrap()
            .kind,
        WorkflowErrorKind::UnsafeStaging
    );
    let linked = temp.path().join("linked-root");
    symlink(temp.path(), &linked).unwrap();
    assert_eq!(
        verify(
            &linked,
            &fixture.response,
            ProviderResourceLimits::default(),
        )
        .err()
        .unwrap()
        .kind,
        WorkflowErrorKind::UnsafeStaging
    );
}

#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn staging_rejects_non_utf8_entries() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let temp = tempfile::tempdir().unwrap();
    let fixture = fixture();
    let root = temp.path().join("entries");
    fs::create_dir(&root).unwrap();
    fs::write(root.join(&fixture.payload_name), b"speech").unwrap();
    fs::write(root.join(OsString::from_vec(vec![0xff])), b"x").unwrap();
    assert_eq!(
        verify(&root, &fixture.response, ProviderResourceLimits::default(),)
            .err()
            .unwrap()
            .kind,
        WorkflowErrorKind::UnsafeStaging
    );
}

fn verify(
    root: &std::path::Path,
    response: &veac_provider::ProviderResponseEnvelope,
    limits: ProviderResourceLimits,
) -> super::super::WorkflowResult<Vec<staging::VerifiedPayload>> {
    staging::verify(
        root,
        response,
        limits,
        Instant::now() + Duration::from_secs(10),
    )
}
