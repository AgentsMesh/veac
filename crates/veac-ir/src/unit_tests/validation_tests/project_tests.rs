use std::error::Error;

use crate::test_support::time;

use super::*;

#[test]
fn envelope_and_project_guards_are_structured() {
    let mut project = sample_project();
    project.schema = "wrong".to_owned();
    project.schema_version = 99;
    project.min_reader_version = 0;
    project.project.id = serde_json::from_str("\"bad\"").unwrap();
    project.project.timebase = 0;
    project.project.revision = MAX_SAFE_INTEGER + 1;
    project.project.entry_sequence_id = SequenceId::new("seq_missing").unwrap();
    let operation = AppliedOperation {
        id: OperationId::new("op_duplicate").unwrap(),
        request_hash: "0".repeat(64),
    };
    project.project.applied_operations = vec![
        operation.clone(),
        operation,
        AppliedOperation {
            id: OperationId::new("op_bad_hash").unwrap(),
            request_hash: "XYZ".to_owned(),
        },
    ];

    let errors = validate(&project).unwrap_err();
    assert!(errors.to_string().contains("diagnostic(s); first is"));
    assert!(errors.source().is_none());
    assert!(!errors.diagnostics().is_empty());
    let codes: Vec<_> = errors
        .into_diagnostics()
        .into_iter()
        .map(|error| error.code)
        .collect();
    for code in [
        "SCHEMA_ID",
        "SCHEMA_VERSION",
        "MIN_READER_VERSION",
        "INVALID_ID",
        "TIMEBASE",
        "REVISION_RANGE",
        "ENTRY_SEQUENCE_NOT_FOUND",
        "DUPLICATE_OPERATION_ID",
        "OPERATION_HASH",
    ] {
        assert_code(&codes, code);
    }
}

#[test]
fn material_locator_identity_probe_and_intent_are_strict() {
    let mut project = sample_project();
    let material = &mut project.project.materials[0];
    material.source = MaterialSource::File {
        uri: "../outside.mp4".to_owned(),
    };
    material.identity = Some(MediaIdentity {
        algorithm: HashAlgorithm::Blake3,
        digest: String::new(),
    });
    material.stream_intent.video = StreamChoice::Disabled;
    let probe = material.probe.as_mut().unwrap();
    probe.container_duration = Some(RationalTime::new(-1, 600).unwrap());
    probe.observed_identity.digest = "different".to_owned();
    probe.streams[0].disposition.attached_picture = true;
    probe.streams[1].type_index = 4;
    probe.streams[1].audio.as_mut().unwrap().sample_rate = 0;

    let mut remote = project.project.materials[1].clone();
    remote.id = MaterialId::new("med_remote").unwrap();
    remote.source = MaterialSource::Remote {
        uri: "https://".to_owned(),
    };
    project.project.materials.push(remote);
    let mut duplicate = project.project.materials[1].clone();
    duplicate.id = project.project.materials[0].id.clone();
    project.project.materials.push(duplicate);

    let codes = validation_codes(&project);
    for code in [
        "MATERIAL_URI",
        "MATERIAL_IDENTITY",
        "STREAM_INTENT",
        "PROBE_DURATION",
        "PROBE_IDENTITY",
        "PROBE_STREAM_ORDER",
        "PROBE_STREAM_FACTS",
        "PROBE_KIND",
        "PROBE_INTENT",
        "DUPLICATE_MATERIAL_ID",
    ] {
        assert_code(&codes, code);
    }

    let mut remote_ok = sample_project();
    remote_ok.project.materials[0].source = MaterialSource::Remote {
        uri: "https://cdn.example/video.mp4".to_owned(),
    };
    validate(&remote_ok).unwrap();
    remote_ok.project.materials[0].source = MaterialSource::Remote {
        uri: "http://localhost:8080/video.mp4".to_owned(),
    };
    validate(&remote_ok).unwrap();
    assert_eq!(time(0).timescale, 600);
}

#[test]
fn canonical_material_identity_rejects_blake3_even_with_a_sha256_shaped_digest() {
    let mut project = sample_project();
    project.project.materials[0].identity = Some(MediaIdentity {
        algorithm: HashAlgorithm::Blake3,
        digest: "a".repeat(64),
    });
    assert!(validation_codes(&project)
        .iter()
        .any(|code| code == "MATERIAL_IDENTITY"));
}

#[test]
fn output_contract_rejects_bad_refs_geometry_paths_audio_and_ids() {
    let mut project = sample_project();
    let output = &mut project.project.render_configs[0];
    output.id = serde_json::from_str("\"bad\"").unwrap();
    output.sequence_id = SequenceId::new("seq_missing").unwrap();
    output.deliverables[0].target = DeliverableTarget::File {
        name: "..\\escape.mp4".to_owned(),
    };
    let raster = output.raster.as_mut().unwrap();
    raster.width = 0;
    raster.frame_rate = Rational {
        numerator: 60,
        denominator: 2,
    };
    output
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Opus,
        sample_rate: 0,
        channels: 33,
    });
    let duplicate = output.clone();
    project.project.render_configs.push(duplicate);
    let codes = validation_codes(&project);
    for code in [
        "INVALID_ID",
        "DUPLICATE_OUTPUT_ID",
        "OUTPUT_SEQUENCE_NOT_FOUND",
        "OUTPUT_GEOMETRY",
        "OUTPUT_TARGET",
        "OUTPUT_AUDIO",
    ] {
        assert_code(&codes, code);
    }
}
