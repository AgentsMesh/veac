use super::{contract_error, parameters, publish};
use crate::{
    CancellationToken, ContentDigest, ExecuteRequest, NodeCacheKey, NodeId, ProjectAction,
    ProjectComputation, ProjectFileSnapshot, ProjectOutputSemantics, ProjectSourceGraphRevision,
};
use veac_artifact::{ArtifactEvidenceOutcome, ArtifactParameters, ArtifactStore};
use veac_project::{
    MatrixAssignment, MediaType, OutputId, ProjectOutput, TargetId, TargetInstanceId,
};

fn action(output: ProjectOutput) -> ProjectAction {
    ProjectAction::VeacRender {
        computation: ProjectComputation {
            instance: TargetInstanceId::from("instance"),
            target: TargetId::from("target"),
            profile: None,
            locale: None,
            matrix: MatrixAssignment::new(),
            inputs: Vec::new(),
            outputs: vec![output],
            bound_sources: Vec::new(),
            package_mounts: Vec::new(),
        },
        source: ProjectFileSnapshot {
            path: "main.veac".to_owned(),
            content: ContentDigest::sha256(b"source"),
            size_bytes: 6,
        },
        source_graph: ProjectSourceGraphRevision {
            root_module: "main.veac".to_owned(),
            authored_source_graph_sha256: "0".repeat(64),
            complete_source_graph_sha256: "1".repeat(64),
            authored_module_count: 1,
            authored_modules: vec!["main.veac".to_owned()],
        },
    }
}

#[test]
fn evidence_semantics_are_closed_and_action_specific() {
    let output = ProjectOutput::Directory {
        id: OutputId::from("evidence"),
    };
    let semantics = ProjectOutputSemantics::Evidence {
        suite_sha256: "1".repeat(64),
        report_sha256: "2".repeat(64),
        outcome: ArtifactEvidenceOutcome::Pass,
    };
    let evidence = ProjectAction::Evidence {
        computation: action(output.clone()).computation().clone(),
        contract: ProjectFileSnapshot {
            path: "contract.veac".to_owned(),
            content: ContentDigest::sha256(b"contract"),
            size_bytes: 8,
        },
        source_graph: ProjectSourceGraphRevision {
            root_module: "contract.veac".to_owned(),
            authored_source_graph_sha256: "3".repeat(64),
            complete_source_graph_sha256: "4".repeat(64),
            authored_module_count: 1,
            authored_modules: vec!["contract.veac".to_owned()],
        },
    };
    assert!(matches!(
        parameters(&evidence, &output, &semantics, 0).unwrap(),
        ArtifactParameters::EvidenceBundle(_)
    ));
    assert!(parameters(
        &action(output),
        evidence.computation().outputs.first().unwrap(),
        &semantics,
        0
    )
    .unwrap_err()
    .message()
    .contains("semantics"));
    assert!(contract_error(std::io::Error::other("contract"))
        .message()
        .contains("contract"));
    assert!(contract_error(crate::BuildError::invalid("build contract"))
        .message()
        .contains("build contract"));
}

#[test]
fn publication_maps_artifact_store_failures() {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    std::fs::create_dir(&workspace).unwrap();
    std::fs::write(workspace.join("video.bin"), b"payload").unwrap();
    let store_root = temp.path().join("store-file");
    std::fs::write(&store_root, b"x").unwrap();
    let output = ProjectOutput::Media {
        id: OutputId::from("video"),
        media_type: MediaType::Video,
    };
    let action = action(output);
    let node = NodeId::new("node").unwrap();
    let key =
        NodeCacheKey::computation("test", 1, ContentDigest::sha256(b"configuration")).unwrap();
    let request = ExecuteRequest {
        node_id: &node,
        action: &action,
        inputs: &[],
        cache_key: &key,
    };
    let produced = [crate::ProducedProjectOutput {
        output: OutputId::from("video"),
        relative_path: "video.bin".into(),
        semantics: ProjectOutputSemantics::Opaque,
    }];
    let error = publish(
        &ArtifactStore::new(store_root),
        request,
        &workspace,
        &produced,
        &CancellationToken::new(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), crate::ExecutionErrorKind::Failed);
    assert!(error.message().contains("cache"), "{error}");
}
