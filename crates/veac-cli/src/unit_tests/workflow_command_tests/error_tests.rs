#![cfg(unix)]

use std::io::Write;
use std::time::Duration;

use veac_artifact::{ArtifactCommitState, ArtifactStore, OwnedStagedFile};
use veac_runtime::workflow::{ProviderResourceLimits, ProviderRunner, WorkflowError};

use crate::arguments::DeriveArgs;

use super::support::{
    artifact_request_bytes, executable, fake_ffprobe, fake_provider, provider_fixture,
};

#[test]
fn workflow_resource_limits_remain_typed_cli_errors() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = provider_fixture();
    let limits = ProviderResourceLimits {
        max_wall_time: Duration::from_nanos(1),
        ..ProviderResourceLimits::default()
    };
    let error = ProviderRunner::new(fake_provider(temp.path(), &fixture))
        .with_limits(limits)
        .run(
            &ArtifactStore::new(temp.path().join("store")),
            &fixture.request,
        )
        .unwrap_err();

    let error =
        crate::commands::workflow_io::workflow::<()>(Err(error), "WORKFLOW_LIMIT").unwrap_err();
    assert!(error.is_resource_limit());
    assert!(error.to_string().contains("pin provider executable"));
}

#[test]
fn workflow_cli_errors_show_committed_artifact_warnings_from_their_source_chain() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("published");
    let mut staged = OwnedStagedFile::new_in(temp.path()).unwrap();
    staged.file_mut().write_all(b"payload").unwrap();
    staged.seal().unwrap();
    let artifact = staged
        .persist_noclobber_while(&destination, || !destination.exists())
        .unwrap_err();
    assert_eq!(artifact.commit_state, ArtifactCommitState::Committed);

    let workflow = WorkflowError::from(artifact);
    let error =
        crate::commands::workflow_io::workflow::<()>(Err(workflow), "WORKFLOW_COMMIT").unwrap_err();
    let message = error.to_string();
    assert!(message.contains("artifact store operation failed"));
    assert!(message.contains("publication crossed the commit point"));
    assert!(message.contains("do not retry blindly"));
    assert!(!error.is_resource_limit());
}

#[test]
fn derive_preserves_ffmpeg_resource_limits_as_typed_cli_errors() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input.bin");
    let request = temp.path().join("request.json");
    let diagnostics = temp.path().join("diagnostics.bin");
    std::fs::write(&input, b"source").unwrap();
    std::fs::write(&diagnostics, vec![b'x'; 2 * 1024 * 1024]).unwrap();
    let ffmpeg = executable(
        temp.path(),
        "limited-ffmpeg.sh",
        &format!(
            "#!/bin/sh\nif [ \"$1\" = -version ]; then echo 'ffmpeg version limited'; exit 0; fi\n/bin/cat '{}' >&2\n",
            diagnostics.display()
        ),
    );
    std::fs::write(&request, artifact_request_bytes(&ffmpeg)).unwrap();

    let error = crate::commands::derive(DeriveArgs {
        input,
        spec: request,
        store: temp.path().join("store"),
        ffmpeg,
        ffprobe: fake_ffprobe(temp.path(), 0),
    })
    .unwrap_err();
    assert!(error.is_resource_limit());
    assert!(error
        .to_string()
        .contains("FFmpeg stderr exceeded the execution budget"));
}
