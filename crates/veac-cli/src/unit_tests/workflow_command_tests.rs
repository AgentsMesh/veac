#[path = "workflow_command_tests/error_tests.rs"]
mod error_tests;
#[path = "workflow_command_tests/support.rs"]
mod support;

use crate::arguments::{DeriveArgs, ProviderProposeArgs, ProviderRunArgs};
use support::*;

#[cfg(unix)]
#[test]
fn workflow_commands_derive_run_provider_and_build_atomic_proposal() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input.bin");
    let artifact_request = temp.path().join("artifact-request.json");
    let ffmpeg = fake_ffmpeg(temp.path());
    let ffprobe = fake_ffprobe(temp.path(), 0);
    std::fs::write(&input, b"source").unwrap();
    std::fs::write(&artifact_request, artifact_request_bytes(&ffmpeg)).unwrap();
    crate::commands::derive(DeriveArgs {
        input,
        spec: artifact_request,
        store: temp.path().join("artifact-store"),
        ffmpeg,
        ffprobe,
    })
    .unwrap();

    let fixture = provider_fixture();
    let request = temp.path().join("provider-request.json");
    let response = temp.path().join("provider-response.json");
    std::fs::write(
        &request,
        veac_provider::canonical_request_bytes(&fixture.request).unwrap(),
    )
    .unwrap();
    crate::commands::provider_run(ProviderRunArgs {
        request: request.clone(),
        program: fake_provider(temp.path(), &fixture),
        store: temp.path().join("provider-store"),
        arguments: vec!["literal".into()],
        response: Some(response.clone()),
    })
    .unwrap();
    let executed: veac_provider::ProviderResponseEnvelope =
        serde_json::from_slice(&std::fs::read(&response).unwrap()).unwrap();
    assert_eq!(executed, fixture.response);

    let (project, context) = caption_project(&temp);
    let context_file = temp.path().join("context.json");
    let proposal_file = temp.path().join("proposal.json");
    std::fs::write(&context_file, serde_json::to_vec(&context).unwrap()).unwrap();
    crate::commands::provider_propose(ProviderProposeArgs {
        project,
        request,
        response,
        context: context_file,
        output: Some(proposal_file.clone()),
    })
    .unwrap();
    let proposal: veac_provider::ProviderEditProposal =
        serde_json::from_slice(&std::fs::read(proposal_file).unwrap()).unwrap();
    assert!(proposal.batch.atomic);
    assert!(matches!(
        &proposal.evidence[0],
        veac_provider::ProposalEvidence::AsrSegment { segment_id, .. }
            if segment_id == "segment-1"
    ));
}

#[test]
fn workflow_json_rejects_duplicates_invalid_utf8_and_output_aliases() {
    let temp = tempfile::tempdir().unwrap();
    let duplicate = temp.path().join("duplicate.json");
    std::fs::write(&duplicate, br#"{"x":1,"x":2}"#).unwrap();
    assert!(
        crate::commands::workflow_io::read_json::<serde_json::Value>(&duplicate, "fixture")
            .unwrap_err()
            .to_string()
            .contains("WORKFLOW_JSON")
    );
    assert!(crate::commands::workflow_io::write(&[0xff], None, &[]).is_err());
    let protected = temp.path().join("protected.json");
    std::fs::write(&protected, "protected").unwrap();
    assert!(crate::commands::workflow_io::write(
        &crate::commands::workflow_io::canonical(&serde_json::json!({"ok": true})).unwrap(),
        Some(&protected),
        std::slice::from_ref(&protected),
    )
    .is_err());

    let malformed = temp.path().join("malformed.json");
    std::fs::write(&malformed, "{}").unwrap();
    assert!(
        crate::commands::workflow_io::read_json::<veac_artifact::MediaArtifactRequest>(
            &malformed, "fixture"
        )
        .unwrap_err()
        .to_string()
        .contains("invalid fixture")
    );
    assert!(crate::commands::workflow_io::canonical(&CannotSerialize)
        .unwrap_err()
        .to_string()
        .contains("WORKFLOW_JSON"));
    let oversized = temp.path().join("oversized.json");
    std::fs::write(&oversized, vec![b' '; 11]).unwrap();
    assert!(
        crate::commands::workflow_io::read_json_bounded::<serde_json::Value>(
            &oversized, "fixture", 10,
        )
        .unwrap_err()
        .to_string()
        .contains("READ_FAILED")
    );
}

#[cfg(unix)]
#[test]
fn workflow_process_and_revision_failures_keep_stable_error_codes() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input.bin");
    let artifact_request = temp.path().join("artifact-request.json");
    let ffmpeg = failing_program(temp.path());
    let ffprobe = fake_ffprobe(temp.path(), 0);
    std::fs::write(&input, b"source").unwrap();
    std::fs::write(&artifact_request, artifact_request_bytes(&ffmpeg)).unwrap();
    let error = crate::commands::derive(DeriveArgs {
        input,
        spec: artifact_request,
        store: temp.path().join("artifact-store"),
        ffmpeg,
        ffprobe,
    })
    .unwrap_err();
    assert!(error.to_string().contains("DERIVE_FAILED"));

    let fixture = provider_fixture();
    let request = temp.path().join("request.json");
    let response = temp.path().join("response.json");
    std::fs::write(
        &request,
        veac_provider::canonical_request_bytes(&fixture.request).unwrap(),
    )
    .unwrap();
    let error = crate::commands::provider_run(ProviderRunArgs {
        request: request.clone(),
        program: failing_program(temp.path()),
        store: temp.path().join("provider-store"),
        arguments: vec![],
        response: None,
    })
    .unwrap_err();
    assert!(error.to_string().contains("PROVIDER_RUN_FAILED"));

    std::fs::write(
        &response,
        veac_provider::canonical_response_bytes(&fixture.response).unwrap(),
    )
    .unwrap();
    let (project, mut context) = caption_project(&temp);
    let veac_provider::ApplicationContext::AsrCaptions(value) = &mut context else {
        unreachable!("caption fixture must use an ASR application context");
    };
    value.header.project_revision += 1;
    let context_file = temp.path().join("context.json");
    std::fs::write(&context_file, serde_json::to_vec(&context).unwrap()).unwrap();
    let error = crate::commands::provider_propose(ProviderProposeArgs {
        project,
        request,
        response,
        context: context_file,
        output: None,
    })
    .unwrap_err();
    assert!(error.to_string().contains("PROVIDER_PROPOSAL_FAILED"));
}

struct CannotSerialize;

impl serde::Serialize for CannotSerialize {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(serde::ser::Error::custom("serialization failed"))
    }
}
