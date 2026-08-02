use tempfile::tempdir;

use super::support::*;
use crate::arguments::{CaptionCommand, CaptionFormatArg, CaptionOverlapArg};

#[test]
fn caption_interchange_fails_closed_on_loss_json_and_paths() {
    let temp = tempdir().unwrap();
    let malformed = temp.path().join("bad.srt");
    std::fs::write(&malformed, "not subtitles").unwrap();
    assert!(run(import_command(&malformed, None))
        .unwrap_err()
        .to_string()
        .contains("CAPTION_CONTRACT"));

    let sidecar = temp.path().join("caption.srt");
    std::fs::write(&sidecar, "1\n00:00:00,000 --> 00:00:01,000\nHello\n").unwrap();
    assert!(run(import_command(&sidecar, Some(sidecar.clone())))
        .unwrap_err()
        .to_string()
        .contains("OUTPUT_OVERWRITES_INPUT"));

    let duplicate = temp.path().join("duplicate.json");
    std::fs::write(&duplicate, r#"{"schema":"x","schema":"y"}"#).unwrap();
    assert!(run(CaptionCommand::Export {
        document: duplicate,
        format: CaptionFormatArg::Srt,
        output: None,
        loss_report: None,
        allow_lossy: false,
    })
    .unwrap_err()
    .to_string()
    .contains("CAPTION_CONTRACT"));
}

#[test]
fn lossy_caption_export_requires_a_file_backed_report() {
    let temp = tempdir().unwrap();
    let sidecar = temp.path().join("caption.vtt");
    let document = temp.path().join("caption.json");
    std::fs::write(
        &sidecar,
        "WEBVTT\n\n00:00:00.000 --> 00:00:01.000\n<v Speaker>Hello\n",
    )
    .unwrap();
    run(CaptionCommand::Import {
        input: sidecar,
        format: CaptionFormatArg::WebVtt,
        timescale: 1000,
        overlap: CaptionOverlapArg::Allow,
        namespace: "voice".to_owned(),
        output: Some(document.clone()),
        loss_report: None,
        allow_lossy: false,
    })
    .unwrap();
    let export = |loss_report, allow_lossy| CaptionCommand::Export {
        document: document.clone(),
        format: CaptionFormatArg::Srt,
        output: Some(temp.path().join("caption-out.srt")),
        loss_report,
        allow_lossy,
    };
    assert!(run(export(None, false))
        .unwrap_err()
        .to_string()
        .contains("CAPTION_LOSS_UNACKNOWLEDGED"));
    assert!(run(export(Some("-".into()), true))
        .unwrap_err()
        .to_string()
        .contains("CAPTION_LOSS_PATH"));
}

#[test]
fn caption_project_commands_reject_invalid_targets_and_bindings() {
    let temp = tempdir().unwrap();
    let sidecar = temp.path().join("caption.srt");
    let document = temp.path().join("caption.json");
    std::fs::write(&sidecar, "1\n00:00:00,000 --> 00:00:01,000\nHello\n").unwrap();
    run(import_command(&sidecar, Some(document.clone()))).unwrap();
    let (project, insertion, _, _) = project_and_bindings(&temp, &document);
    assert!(run(CaptionCommand::Propose {
        project: project.clone(),
        document: document.clone(),
        bindings: insertion.clone(),
        operation_id: "invalid".to_owned(),
        output: None,
    })
    .unwrap_err()
    .to_string()
    .contains("CAPTION_BINDING_ID"));

    let mut rejected: veac_caption::CaptionTrackInsertionBindings =
        serde_json::from_slice(&std::fs::read(&insertion).unwrap()).unwrap();
    rejected.sequence_id = veac_ir::SequenceId::new("seq_missing").unwrap();
    std::fs::write(&insertion, serde_json::to_vec(&rejected).unwrap()).unwrap();
    assert!(run(CaptionCommand::Propose {
        project: project.clone(),
        document: document.clone(),
        bindings: insertion,
        operation_id: "op_missing_sequence".to_owned(),
        output: None,
    })
    .unwrap_err()
    .to_string()
    .contains("EDIT_REJECTED"));

    let bad_bindings = temp.path().join("bad-bindings.json");
    std::fs::write(&bad_bindings, r#"{"cue_ids":{},"cue_ids":{}}"#).unwrap();
    assert!(run(CaptionCommand::Extract {
        project: project.clone(),
        track: "trk_missing".to_owned(),
        bindings: bad_bindings.clone(),
        output: None,
    })
    .unwrap_err()
    .to_string()
    .contains("CAPTION_TRACK_MISSING"));
    assert!(run(CaptionCommand::Extract {
        project,
        track: "not-an-id".to_owned(),
        bindings: bad_bindings,
        output: None,
    })
    .unwrap_err()
    .to_string()
    .contains("CAPTION_BINDING_ID"));

    let malformed = temp.path().join("malformed-bindings.json");
    std::fs::write(&malformed, "{]").unwrap();
    assert!(run(CaptionCommand::Extract {
        project: project_and_bindings(&temp, &document).0,
        track: "trk_base".to_owned(),
        bindings: malformed,
        output: None,
    })
    .unwrap_err()
    .to_string()
    .contains("CAPTION_JSON"));
}
